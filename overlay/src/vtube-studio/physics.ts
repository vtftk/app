export type PhysicsObject = {
  // Current position of the object
  x: number;
  y: number;

  // Velocity of the object
  velocityX: number;
  velocityY: number;

  // HTML element the physics are applied to
  movement: HTMLElement;
};

export type PhysicsWorld = {
  objects: PhysicsObject[];
};

export type PhysicsEngine = {
  pushObject: (object: PhysicsObject) => void;
  config: PhysicsEngineConfig;
  stop: VoidFunction;
};

export type PhysicsEngineConfig = {
  // Frames per second to run the physics engine at (60)
  fps: number;

  // Multiplier applied against gravity (1)
  gravityMultiplier: number;
};

export function createPhysicsEngine(config: PhysicsEngineConfig) {
  const { fps, gravityMultiplier } = config;

  const frameTime = 1000 / fps;
  const deltaFrameTime = 1 / fps;
  const gravityConstant = 30;
  const reverseGravity = gravityMultiplier < 0;

  const world: PhysicsWorld = { objects: [] };

  let ticking = false;
  let lastTime: DOMHighResTimeStamp = 0;
  let accumulatedTime: number = 0;

  function tick(timestamp: DOMHighResTimeStamp) {
    if (!ticking) return;

    if (!lastTime) lastTime = timestamp;

    // Calculate time elapsed since the last frame
    const deltaTime = timestamp - lastTime;
    lastTime = timestamp;

    // Accumulate time
    accumulatedTime += deltaTime;

    const objects = world.objects;

    // Process animation frames (Update positions)
    while (accumulatedTime >= frameTime) {
      const objectsLength = objects.length;

      for (let i = 0; i < objectsLength; i++) {
        const object = objects[i];

        // Apply gravity
        object.velocityY +=
          gravityConstant * gravityMultiplier * deltaFrameTime;

        // Apply velocity
        object.y += object.velocityY;
        object.x += object.velocityX;
      }

      accumulatedTime -= frameTime;
    }

    // Apply animation position to elements
    for (let i = 0; i < objects.length; i++) {
      const object = objects[i];

      // Remove objects that have left the screen
      if (object.y > 100 || (reverseGravity && object.y < -100)) {
        document.body.removeChild(object.movement);
        objects.splice(i--, 1);
        continue;
      }

      // Apply current object position translation
      object.movement.style.transform = `translate(${object.x}vw, ${object.y}vh)`;
    }

    if (objects.length > 0 && ticking) {
      requestAnimationFrame(tick);
    } else {
      // Nothing left to tick, stop scheduling frames
      ticking = false;
    }
  }

  return {
    pushObject: (object: PhysicsObject) => {
      // Add object to the world
      world.objects.push(object);

      // Schedule ticking if not currently ticking world objects
      if (!ticking) {
        ticking = true;
        requestAnimationFrame(tick);
      }
    },

    config,

    // Stop all ticking and clear the world objects
    stop: () => {
      ticking = false;
      world.objects = [];
    },
  };
}
