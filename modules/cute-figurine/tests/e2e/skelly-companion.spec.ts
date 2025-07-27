import { test, expect } from '@playwright/test';

test.describe('Skelly Companion E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/');
  });

  test('should render the Skelly companion', async ({ page }) => {
    // Look for the companion container
    const companion = page.getByRole('img', { name: 'Skelly the companion' });
    await expect(companion).toBeVisible();
  });

  test('should have proper accessibility attributes', async ({ page }) => {
    const companion = page.getByRole('img', { name: 'Skelly the companion' });
    
    // Check aria attributes
    await expect(companion).toHaveAttribute('aria-live', 'polite');
    await expect(companion).toHaveAttribute('tabindex', '0');
  });

  test('should be draggable', async ({ page }) => {
    const companion = page.getByRole('img', { name: 'Skelly the companion' });
    
    // Get initial position
    const initialBox = await companion.boundingBox();
    expect(initialBox).not.toBeNull();
    
    // Perform drag operation
    await companion.dragTo(page.locator('body'), {
      targetPosition: { x: 300, y: 300 }
    });
    
    // Check if position changed
    const newBox = await companion.boundingBox();
    expect(newBox).not.toBeNull();
    expect(newBox!.x).not.toBe(initialBox!.x);
  });

  test('should respond to keyboard navigation', async ({ page }) => {
    const companion = page.getByRole('img', { name: 'Skelly the companion' });
    
    // Focus the companion
    await companion.focus();
    await expect(companion).toBeFocused();
    
    // Test keyboard interaction
    await page.keyboard.press('Enter');
    // Note: Add specific assertions based on expected keyboard behavior
  });

  test('should maintain performance targets', async ({ page }) => {
    // Start performance monitoring
    await page.addInitScript(() => {
      (window as any).performanceMetrics = {
        startTime: performance.now(),
        frameCount: 0
      };
      
      // Monitor frame rate
      function countFrames() {
        (window as any).performanceMetrics.frameCount++;
        requestAnimationFrame(countFrames);
      }
      requestAnimationFrame(countFrames);
    });
    
    // Wait for companion to load
    const companion = page.getByRole('img', { name: 'Skelly the companion' });
    await expect(companion).toBeVisible();
    
    // Wait a bit for animations
    await page.waitForTimeout(2000);
    
    // Check performance metrics
    const metrics = await page.evaluate(() => {
      const perf = (window as any).performanceMetrics;
      const elapsed = (performance.now() - perf.startTime) / 1000;
      const fps = perf.frameCount / elapsed;
      return { fps, elapsed };
    });
    
    // Verify performance targets (should maintain reasonable FPS)
    expect(metrics.fps).toBeGreaterThan(30);
  });

  test('should handle state changes gracefully', async ({ page }) => {
    const companion = page.getByRole('img', { name: 'Skelly the companion' });
    await expect(companion).toBeVisible();
    
    // Test state change simulation
    await page.evaluate(() => {
      // Simulate state change through global event or API
      window.dispatchEvent(new CustomEvent('skellyStateChange', {
        detail: { mood: 'excited', energy: 90 }
      }));
    });
    
    // Check for visual feedback (animation changes, etc.)
    // Note: Add specific assertions based on expected state change behavior
    await page.waitForTimeout(1000);
    await expect(companion).toBeVisible();
  });

  test('should be responsive across different viewport sizes', async ({ page }) => {
    // Test desktop size
    await page.setViewportSize({ width: 1920, height: 1080 });
    const companion = page.getByRole('img', { name: 'Skelly the companion' });
    await expect(companion).toBeVisible();
    
    // Test tablet size
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(companion).toBeVisible();
    
    // Test mobile size
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(companion).toBeVisible();
  });

  test('should handle errors gracefully', async ({ page }) => {
    // Monitor console errors
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    // Load the companion
    const companion = page.getByRole('img', { name: 'Skelly the companion' });
    await expect(companion).toBeVisible();
    
    // Simulate error conditions
    await page.evaluate(() => {
      // Simulate network error or other edge case
      const event = new Event('error');
      window.dispatchEvent(event);
    });
    
    await page.waitForTimeout(1000);
    
    // Verify no critical errors occurred
    const criticalErrors = errors.filter(error => 
      error.includes('Uncaught') || error.includes('TypeError')
    );
    expect(criticalErrors).toHaveLength(0);
  });
});