//! Module registry and dependency graph management

use crate::error::{OrchestratorError, OrchestratorResult};
use crate::lifecycle::ModuleState;
use dashmap::DashMap;
use skelly_jelly_event_bus::ModuleId;
use petgraph::{Graph, Direction};
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use semver::Version;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{debug, info, warn};

/// Descriptor for a module in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDescriptor {
    pub id: ModuleId,
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<ModuleId>,
    pub required: bool,  // If false, system can run without it
    pub startup_timeout: Duration,
    pub shutdown_timeout: Duration,
    pub health_check_interval: Duration,
}

impl ModuleDescriptor {
    pub fn new(id: ModuleId, name: String) -> Self {
        Self {
            id,
            name,
            version: Version::new(0, 1, 0),
            dependencies: Vec::new(),
            required: true,
            startup_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(10),
            health_check_interval: Duration::from_secs(30),
        }
    }

    pub fn with_dependencies(mut self, dependencies: Vec<ModuleId>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_timeouts(mut self, startup: Duration, shutdown: Duration) -> Self {
        self.startup_timeout = startup;
        self.shutdown_timeout = shutdown;
        self
    }
}

/// Dependency graph for managing module startup order
pub struct DependencyGraph {
    graph: UnGraph<ModuleId, ()>,
    node_indices: HashMap<ModuleId, NodeIndex>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new_undirected(),
            node_indices: HashMap::new(),
        }
    }

    /// Add a module to the dependency graph
    pub fn add_module(&mut self, module_id: ModuleId) {
        if !self.node_indices.contains_key(&module_id) {
            let node_index = self.graph.add_node(module_id);
            self.node_indices.insert(module_id, node_index);
        }
    }

    /// Add a dependency relationship (dependent depends on dependency)
    pub fn add_dependency(&mut self, dependent: ModuleId, dependency: ModuleId) {
        self.add_module(dependent);
        self.add_module(dependency);

        let dependent_index = self.node_indices[&dependent];
        let dependency_index = self.node_indices[&dependency];

        // Add edge from dependency to dependent (dependency must start first)
        self.graph.add_edge(dependency_index, dependent_index, ());
    }

    /// Compute startup order using topological sort
    pub fn compute_startup_order(&self) -> OrchestratorResult<Vec<ModuleId>> {
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut result = Vec::new();

        // Convert to directed graph for topological sort
        for &node_index in self.node_indices.values() {
            if !visited.contains(&node_index) {
                self.topological_sort_visit(
                    node_index,
                    &mut visited,
                    &mut temp_visited,
                    &mut result,
                )?;
            }
        }

        result.reverse();
        Ok(result.into_iter().map(|idx| self.graph[idx]).collect())
    }

    /// Recursive helper for topological sort with cycle detection
    fn topological_sort_visit(
        &self,
        node: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        temp_visited: &mut HashSet<NodeIndex>,
        result: &mut Vec<NodeIndex>,
    ) -> OrchestratorResult<()> {
        if temp_visited.contains(&node) {
            // Cycle detected
            let cycle: Vec<ModuleId> = temp_visited
                .iter()
                .map(|&idx| self.graph[idx])
                .collect();
            return Err(OrchestratorError::DependencyCycle { cycle });
        }

        if visited.contains(&node) {
            return Ok(());
        }

        temp_visited.insert(node);

        // Visit all neighbors (dependencies)
        for edge in self.graph.edges_directed(node, Direction::Outgoing) {
            let neighbor = edge.target();
            self.topological_sort_visit(neighbor, visited, temp_visited, result)?;
        }

        temp_visited.remove(&node);
        visited.insert(node);
        result.push(node);

        Ok(())
    }

    /// Get direct dependencies of a module
    pub fn get_dependencies(&self, module_id: ModuleId) -> Vec<ModuleId> {
        if let Some(&node_index) = self.node_indices.get(&module_id) {
            self.graph
                .edges_directed(node_index, Direction::Incoming)
                .map(|edge| self.graph[edge.source()])
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get modules that depend on this module
    pub fn get_dependents(&self, module_id: ModuleId) -> Vec<ModuleId> {
        if let Some(&node_index) = self.node_indices.get(&module_id) {
            self.graph
                .edges_directed(node_index, Direction::Outgoing)
                .map(|edge| self.graph[edge.target()])
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// Handle for controlling a module
#[derive(Debug)]
pub struct ModuleHandle {
    pub module_id: ModuleId,
    pub start_time: Option<Instant>,
    pub task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ModuleHandle {
    pub fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
            start_time: None,
            task_handle: None,
        }
    }

    pub fn set_started(&mut self, task_handle: tokio::task::JoinHandle<()>) {
        self.start_time = Some(Instant::now());
        self.task_handle = Some(task_handle);
    }

    pub fn is_running(&self) -> bool {
        self.task_handle
            .as_ref()
            .map(|handle| !handle.is_finished())
            .unwrap_or(false)
    }

    pub async fn stop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
            // In a real implementation, we'd send a graceful shutdown signal first
        }
        self.start_time = None;
    }
}

/// Registry for all modules in the system
pub struct ModuleRegistry {
    /// All registered modules
    modules: DashMap<ModuleId, ModuleDescriptor>,
    
    /// Dependency graph for startup ordering
    dependency_graph: Arc<tokio::sync::RwLock<DependencyGraph>>,
    
    /// Module state tracking
    module_states: Arc<DashMap<ModuleId, ModuleState>>,
    
    /// Module handles for lifecycle control
    module_handles: DashMap<ModuleId, ModuleHandle>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            modules: DashMap::new(),
            dependency_graph: Arc::new(tokio::sync::RwLock::new(DependencyGraph::new())),
            module_states: Arc::new(DashMap::new()),
            module_handles: DashMap::new(),
        };

        // Register default modules with their dependencies
        registry.register_default_modules();
        registry
    }

    /// Register a module with the registry
    pub async fn register_module(&self, descriptor: ModuleDescriptor) -> OrchestratorResult<()> {
        let module_id = descriptor.id;
        
        // Check if all dependencies are available
        for &dependency in &descriptor.dependencies {
            if !self.modules.contains_key(&dependency) {
                warn!("Dependency {} for module {} is not yet registered", dependency, module_id);
            }
        }

        // Add to dependency graph
        {
            let mut graph = self.dependency_graph.write().await;
            graph.add_module(module_id);
            for &dependency in &descriptor.dependencies {
                graph.add_dependency(module_id, dependency);
            }
        }

        // Register module
        self.modules.insert(module_id, descriptor);
        self.module_states.insert(module_id, ModuleState::NotStarted);
        self.module_handles.insert(module_id, ModuleHandle::new(module_id));

        info!("Registered module: {}", module_id);
        Ok(())
    }

    /// Get module descriptor
    pub fn get_module(&self, module_id: ModuleId) -> Option<ModuleDescriptor> {
        self.modules.get(&module_id).map(|entry| entry.clone())
    }

    /// Get all registered modules
    pub fn get_all_modules(&self) -> Vec<ModuleDescriptor> {
        self.modules.iter().map(|entry| entry.clone()).collect()
    }

    /// Get module state
    pub fn get_module_state(&self, module_id: ModuleId) -> Option<ModuleState> {
        self.module_states.get(&module_id).map(|entry| entry.clone())
    }

    /// Update module state
    pub fn set_module_state(&self, module_id: ModuleId, state: ModuleState) {
        self.module_states.insert(module_id, state);
        debug!("Module {} state changed to {:?}", module_id, self.module_states.get(&module_id));
    }

    /// Get module handle
    pub fn get_module_handle(&self, module_id: ModuleId) -> Option<dashmap::mapref::one::Ref<ModuleId, ModuleHandle>> {
        self.module_handles.get(&module_id)
    }

    /// Get mutable module handle
    pub fn get_module_handle_mut(&self, module_id: ModuleId) -> Option<dashmap::mapref::one::RefMut<ModuleId, ModuleHandle>> {
        self.module_handles.get_mut(&module_id)
    }

    /// Compute startup order
    pub async fn compute_startup_order(&self) -> OrchestratorResult<Vec<ModuleId>> {
        let graph = self.dependency_graph.read().await;
        graph.compute_startup_order()
    }

    /// Get dependencies of a module
    pub async fn get_dependencies(&self, module_id: ModuleId) -> Vec<ModuleId> {
        let graph = self.dependency_graph.read().await;
        graph.get_dependencies(module_id)
    }

    /// Get dependents of a module
    pub async fn get_dependents(&self, module_id: ModuleId) -> Vec<ModuleId> {
        let graph = self.dependency_graph.read().await;
        graph.get_dependents(module_id)
    }

    /// Register default system modules
    fn register_default_modules(&mut self) {
        // Note: This runs synchronously during construction
        // We'll register modules without updating the dependency graph here
        // The actual dependency setup will happen when modules are properly registered

        let modules = vec![
            (ModuleId::Orchestrator, "orchestrator", vec![]),
            (ModuleId::EventBus, "event-bus", vec![]),
            (ModuleId::Storage, "storage", vec![ModuleId::EventBus]),
            (ModuleId::DataCapture, "data-capture", vec![ModuleId::EventBus]),
            (ModuleId::AnalysisEngine, "analysis-engine", vec![ModuleId::EventBus, ModuleId::Storage]),
            (ModuleId::Gamification, "gamification", vec![ModuleId::EventBus, ModuleId::AnalysisEngine]),
            (ModuleId::AiIntegration, "ai-integration", vec![ModuleId::EventBus, ModuleId::Gamification]),
            (ModuleId::CuteFigurine, "cute-figurine", vec![ModuleId::EventBus, ModuleId::AiIntegration]),
        ];

        for (id, name, dependencies) in modules {
            let descriptor = ModuleDescriptor::new(id, name.to_string())
                .with_dependencies(dependencies);
            
            self.modules.insert(id, descriptor);
            self.module_states.insert(id, ModuleState::NotStarted);
            self.module_handles.insert(id, ModuleHandle::new(id));
        }
    }
}