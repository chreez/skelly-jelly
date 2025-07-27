import * as THREE from 'three';

export interface BlendNode {
  id: string;
  name: string;
  weight: number;
  isActive: boolean;
}

export interface BlendParameter {
  name: string;
  value: number;
  min: number;
  max: number;
}

export interface Blend1DNode extends BlendNode {
  type: '1d';
  parameter: string;
  threshold: number;
  animation: string;
}

export interface Blend2DNode extends BlendNode {
  type: '2d';
  parameterX: string;
  parameterY: string;
  positionX: number;
  positionY: number;
  animation: string;
}

export interface BlendStateNode extends BlendNode {
  type: 'state';
  animation: string;
  children: BlendNode[];
}

export type BlendTreeNode = Blend1DNode | Blend2DNode | BlendStateNode;

export class BlendTree {
  private nodes: Map<string, BlendTreeNode> = new Map();
  private parameters: Map<string, BlendParameter> = new Map();
  private rootNodes: BlendTreeNode[] = [];
  private mixer?: THREE.AnimationMixer;
  private actions: Map<string, THREE.AnimationAction> = new Map();

  constructor() {
    this.setupDefaultParameters();
  }

  private setupDefaultParameters(): void {
    // Common blend parameters for mood/energy blending
    this.addParameter('energy', 0.5, 0, 1);
    this.addParameter('happiness', 0.5, 0, 1);
    this.addParameter('meltLevel', 0, 0, 1);
    this.addParameter('activity', 0, 0, 1); // 0=idle, 0.5=working, 1=celebrating
    this.addParameter('interaction', 0, 0, 1);
  }

  public setMixer(mixer: THREE.AnimationMixer): void {
    this.mixer = mixer;
  }

  public addParameter(name: string, value: number, min: number, max: number): void {
    this.parameters.set(name, { name, value, min, max });
  }

  public setParameter(name: string, value: number): void {
    const param = this.parameters.get(name);
    if (param) {
      param.value = Math.max(param.min, Math.min(param.max, value));
    }
  }

  public getParameter(name: string): number {
    return this.parameters.get(name)?.value || 0;
  }

  public add1DBlendNode(
    id: string,
    name: string,
    animation: string,
    parameter: string,
    threshold: number
  ): Blend1DNode {
    const node: Blend1DNode = {
      id,
      name,
      type: '1d',
      weight: 0,
      isActive: false,
      parameter,
      threshold,
      animation,
    };

    this.nodes.set(id, node);
    return node;
  }

  public add2DBlendNode(
    id: string,
    name: string,
    animation: string,
    parameterX: string,
    parameterY: string,
    positionX: number,
    positionY: number
  ): Blend2DNode {
    const node: Blend2DNode = {
      id,
      name,
      type: '2d',
      weight: 0,
      isActive: false,
      parameterX,
      parameterY,
      positionX,
      positionY,
      animation,
    };

    this.nodes.set(id, node);
    return node;
  }

  public addStateNode(
    id: string,
    name: string,
    animation: string,
    children: BlendNode[] = []
  ): BlendStateNode {
    const node: BlendStateNode = {
      id,
      name,
      type: 'state',
      weight: 0,
      isActive: false,
      animation,
      children,
    };

    this.nodes.set(id, node);
    return node;
  }

  public createDefaultBlendTree(): void {
    // Create a comprehensive blend tree for the companion

    // Idle blend (energy-based)
    const idleNode = this.add1DBlendNode(
      'idle_blend',
      'Idle Blend',
      'idle_breathing',
      'energy',
      0.3
    );

    // Focused blend (energy + focus)
    const focusedNode = this.add2DBlendNode(
      'focused_blend',
      'Focused Blend',
      'focused_breathing',
      'energy',
      'activity',
      0.7,
      0.5
    );

    // Happy/excited blend
    const happyNode = this.add2DBlendNode(
      'happy_blend',
      'Happy Blend',
      'happy_bounce',
      'happiness',
      'energy',
      0.8,
      0.8
    );

    // Tired blend (low energy)
    const tiredNode = this.add1DBlendNode(
      'tired_blend',
      'Tired Blend',
      'tired_sway',
      'energy',
      0.2
    );

    // Melting blend (melt level)
    const meltingLightNode = this.add1DBlendNode(
      'melting_light',
      'Light Melting',
      'melting_medium',
      'meltLevel',
      0.4
    );

    const meltingHeavyNode = this.add1DBlendNode(
      'melting_heavy',
      'Heavy Melting',
      'melting_heavy',
      'meltLevel',
      0.7
    );

    // Celebration blend
    const celebrationNode = this.add2DBlendNode(
      'celebration_blend',
      'Celebration Blend',
      'celebration_dance',
      'happiness',
      'activity',
      1.0,
      1.0
    );

    // Interaction blend
    const interactionNode = this.add1DBlendNode(
      'interaction_blend',
      'Interaction Blend',
      'interaction_wave',
      'interaction',
      0.5
    );

    // Set up root nodes (nodes that can be active simultaneously)
    this.rootNodes = [
      idleNode,
      focusedNode,
      happyNode,
      tiredNode,
      meltingLightNode,
      meltingHeavyNode,
      celebrationNode,
      interactionNode,
    ];
  }

  public update(): void {
    if (!this.mixer) return;

    // Update weights based on parameters
    this.updateNodeWeights();

    // Apply weights to actions
    this.applyWeights();
  }

  private updateNodeWeights(): void {
    // Reset all weights
    this.nodes.forEach((node) => {
      node.weight = 0;
      node.isActive = false;
    });

    // Calculate weights for each node type
    this.rootNodes.forEach((node) => {
      switch (node.type) {
        case '1d':
          this.update1DNode(node);
          break;
        case '2d':
          this.update2DNode(node);
          break;
        case 'state':
          this.updateStateNode(node);
          break;
      }
    });

    // Normalize weights so they sum to 1
    this.normalizeWeights();
  }

  private update1DNode(node: Blend1DNode): void {
    const paramValue = this.getParameter(node.parameter);
    const distance = Math.abs(paramValue - node.threshold);

    // Calculate weight based on distance (closer = higher weight)
    if (distance < 0.3) {
      // Within influence range
      node.weight = Math.max(0, 1 - distance / 0.3);
      node.isActive = node.weight > 0.01;
    }
  }

  private update2DNode(node: Blend2DNode): void {
    const paramX = this.getParameter(node.parameterX);
    const paramY = this.getParameter(node.parameterY);

    const distanceX = Math.abs(paramX - node.positionX);
    const distanceY = Math.abs(paramY - node.positionY);
    const distance = Math.sqrt(distanceX * distanceX + distanceY * distanceY);

    // Calculate weight based on 2D distance
    if (distance < 0.4) {
      // Within influence range
      node.weight = Math.max(0, 1 - distance / 0.4);
      node.isActive = node.weight > 0.01;
    }
  }

  private updateStateNode(node: BlendStateNode): void {
    // State nodes are manually controlled
    // Weight is set externally based on state logic
  }

  private normalizeWeights(): void {
    const activeNodes = Array.from(this.nodes.values()).filter((node) => node.isActive);
    const totalWeight = activeNodes.reduce((sum, node) => sum + node.weight, 0);

    if (totalWeight > 0) {
      activeNodes.forEach((node) => {
        node.weight = node.weight / totalWeight;
      });
    } else {
      // Fallback to idle if no nodes are active
      const idleNode = this.nodes.get('idle_blend');
      if (idleNode) {
        idleNode.weight = 1;
        idleNode.isActive = true;
      }
    }
  }

  private applyWeights(): void {
    if (!this.mixer) return;

    // Apply weights to all active animations
    this.nodes.forEach((node) => {
      if (node.isActive && node.weight > 0) {
        let action = this.actions.get(node.animation);

        if (!action) {
          // Try to get clip from mixer (assuming it was added elsewhere)
          const clips = this.mixer!.getRoot().animations || [];
          const clip = clips.find((c) => c.name === node.animation);

          if (clip) {
            action = this.mixer!.clipAction(clip);
            this.actions.set(node.animation, action);
          }
        }

        if (action) {
          if (!action.isRunning()) {
            action.reset();
            action.play();
          }
          action.weight = node.weight;
        }
      }
    });

    // Stop inactive animations
    this.actions.forEach((action, animationName) => {
      const node = Array.from(this.nodes.values()).find((n) => n.animation === animationName);
      if (!node || !node.isActive || node.weight <= 0.01) {
        if (action.isRunning()) {
          action.weight = 0;
          // Don't stop immediately to allow smooth fade out
          setTimeout(() => {
            if (action.weight === 0) {
              action.stop();
            }
          }, 100);
        }
      }
    });
  }

  public setNodeWeight(nodeId: string, weight: number): void {
    const node = this.nodes.get(nodeId);
    if (node) {
      node.weight = Math.max(0, Math.min(1, weight));
      node.isActive = node.weight > 0.01;
    }
  }

  public getNodeWeight(nodeId: string): number {
    return this.nodes.get(nodeId)?.weight || 0;
  }

  public isNodeActive(nodeId: string): boolean {
    return this.nodes.get(nodeId)?.isActive || false;
  }

  public getActiveNodes(): BlendTreeNode[] {
    return Array.from(this.nodes.values()).filter((node) => node.isActive);
  }

  public getBlendState(): Record<string, number> {
    const state: Record<string, number> = {};

    // Include parameters
    this.parameters.forEach((param, name) => {
      state[`param_${name}`] = param.value;
    });

    // Include active node weights
    this.nodes.forEach((node, id) => {
      if (node.isActive) {
        state[`node_${id}`] = node.weight;
      }
    });

    return state;
  }

  public setBlendState(state: Record<string, number>): void {
    // Restore parameters
    Object.entries(state).forEach(([key, value]) => {
      if (key.startsWith('param_')) {
        const paramName = key.substring(6);
        this.setParameter(paramName, value);
      } else if (key.startsWith('node_')) {
        const nodeId = key.substring(5);
        this.setNodeWeight(nodeId, value);
      }
    });
  }

  public dispose(): void {
    this.actions.forEach((action) => {
      action.stop();
    });

    this.actions.clear();
    this.nodes.clear();
    this.parameters.clear();
    this.rootNodes = [];
    this.mixer = undefined;
  }
}
