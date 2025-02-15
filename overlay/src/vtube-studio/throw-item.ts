import { flinch } from "./flinch";
import { VTubeStudioWebSocket } from "./socket";
import { sleep, LoadedSoundData } from "../utils/async";
import { randomBool, randomRange, percentRange } from "../utils/math";
import { LARGEST_MODEL_SIZE, TOTAL_MODEL_SIZE_RANGE } from "../constants";
import { ModelPosition, ModelParameters, requestCurrentModel } from "./model";
import {
  PhysicsEngine,
  PhysicsEngineConfig,
  createPhysicsEngine,
} from "./physics";
import {
  MinMax,
  AppData,
  ModelId,
  ThrowDirection,
  ItemImageConfig,
  ModelCalibration,
  ItemWithSoundIds,
} from "../vtftk/types";

const HORIZONTAL_PHYSICS_SCALE = 3;
const VERTICAL_PHYSICS_SCALE = 10;

let physicsEngine: PhysicsEngine | null = null;

export function setPhysicsEngineConfig(config: PhysicsEngineConfig) {
  if (physicsEngine !== null) {
    physicsEngine.stop();
    physicsEngine = createPhysicsEngine(config);
  }
}

/**
 * Throws an item
 *
 * @param socket Socket for getting model position and sending impact flinches to VTube studio
 * @param appData Global app data settings
 * @param modelCalibration Calibration data for available models
 * @param modelParameters Parameters for the current model
 * @param config Configuration for the thrown item
 * @param image Image element to use for the thrown item
 * @param impactAudio Audio element to play when the item impacts the target
 * @param windupAudio Audio element to play when the item is winding up
 * @returns Promise that completes after the item has been completely thrown and removed
 */
export async function throwItem(
  socket: VTubeStudioWebSocket,
  appData: AppData,
  modelCalibration: Map<ModelId, ModelCalibration>,
  modelParameters: ModelParameters,
  config: ItemWithSoundIds,
  image: HTMLImageElement,
  impactAudio: LoadedSoundData | null,
  windupAudio: LoadedSoundData | null,
) {
  const { modelID, modelPosition } = await requestCurrentModel(socket);

  const modelData = modelCalibration.get(modelID);

  // Model is not yet calibrated
  if (modelData === undefined) return;

  // Model is not available
  if (!modelPosition) return;

  const { throwables_config: throwables } = appData;

  // Determine scale of the model relative to the calibrated minimum and maximum sizes
  const modelScale =
    (modelPosition.size + LARGEST_MODEL_SIZE) / TOTAL_MODEL_SIZE_RANGE;

  const leftSide: boolean = isRandomDirectionLeft(
    throwables.direction,
    percentRange(modelScale, modelData.x.min, modelData.x.max),
  );

  let angle = randomRange(
    throwables.throw_angle.min,
    throwables.throw_angle.max,
  );

  // Flip the angle when coming from the right side
  if (!leftSide) angle = -angle;

  const itemScale = percentRange(
    modelScale,
    throwables.item_scale.min,
    throwables.item_scale.max,
  );

  const { image: imageConfig, windup } = config.config;

  if (windup.enabled) {
    // Play the windup sound
    if (windupAudio !== null) {
      try {
        windupAudio.sound.volume =
          appData.sounds_config.global_volume * windupAudio.config.volume;
        windupAudio.sound.play();
      } catch (err) {
        console.error("failed to play windup audio", err);
      }
    }

    await sleep(windup.duration);
  }

  const scaledImageWidth = image.width * imageConfig.scale * itemScale;
  const scaledImageHeight = image.height * imageConfig.scale * itemScale;

  const thrown = createThrownImage(
    imageConfig,
    image,
    scaledImageWidth,
    scaledImageHeight,
    angle,
    throwables.spin_speed,
  );

  const movement = createMovementContainer(
    leftSide,
    throwables.duration,
    throwables.impact_delay,
  );

  const pivot = createPivotContainer(
    scaledImageWidth,
    scaledImageHeight,
    modelPosition,
    modelData,
    modelScale,
    angle,
  );

  const root = createRootContainer(modelPosition);

  movement.appendChild(thrown);
  pivot.appendChild(movement);
  root.appendChild(pivot);
  document.body.appendChild(root);

  // Impact is encountered half way through the animation
  const impactTimeout = throwables.duration / 2 + throwables.impact_delay;

  // Wait for the impact to occur
  await sleep(impactTimeout);

  // Handle point of impact
  handleThrowableImpact(
    socket,
    appData,
    modelParameters,
    config,
    impactAudio,
    angle,
    leftSide,
  );

  // No physics to apply
  if (!appData.physics_config.enabled) {
    // Wait remaining duration before removing
    await sleep(throwables.duration / 2);
    // Remove after complete
    document.body.removeChild(root);

    return;
  }

  // Initialize the physics engine
  if (physicsEngine === null) {
    const { fps, gravity_multiplier } = appData.physics_config;

    physicsEngine = createPhysicsEngine({
      fps: fps,
      gravityMultiplier: gravity_multiplier,
    });
  }

  // Strip animations and transforms before applying physics
  movement.style.animationName = "";
  pivot.style.transform = "";
  thrown.style.transform = "";

  const { horizontal_multiplier, vertical_multiplier } = appData.physics_config;

  const randomVelocity = Math.random();

  const velocityX =
    // Apply random velocity
    randomVelocity *
    // Apply world gravity multiplier
    HORIZONTAL_PHYSICS_SCALE *
    // Apply direction velocity
    (leftSide ? -1 : 1) *
    // Apply global velocity multiplier
    horizontal_multiplier;
  const velocityY =
    // Apply random velocity
    (1 - randomVelocity) *
    // Apply world gravity multiplier
    VERTICAL_PHYSICS_SCALE *
    // Apply direction velocity (Up or down)
    (angle < 0 ? -1 : 0.5) *
    // Apply global velocity multiplier
    vertical_multiplier;

  physicsEngine.pushObject({
    x: 0,
    y: 0,
    velocityX,
    velocityY,
    root,
    movement,
  });
}

/**
 * Chooses a direction based on the provided throw direction
 * config returning whether that direction is left
 *
 * @param direction The direction config
 * @param xPos Window model relative position
 * @returns Whether the direction is left
 */
function isRandomDirectionLeft(
  direction: ThrowDirection,
  xPos: number,
): boolean {
  switch (direction) {
    case ThrowDirection.Weighted:
      return Math.random() >= (xPos + 1) / 2;
    case ThrowDirection.Random:
      return randomBool();
    case ThrowDirection.LeftOnly:
      return true;
    case ThrowDirection.RightOnly:
      return false;
    default:
      return false;
  }
}

/**
 * Handles the point of impact for a throwable hitting the model
 *
 * @param socket Socket for sending impact flinches to VTube studio
 * @param appData Global app data settings
 * @param modelParameters Parameters for the current model
 * @param config Configuration for the thrown item
 * @param impactAudio Audio element to play when the item impacts the target
 * @param angle Angle the item was thrown at
 * @param leftSide Whether the item is coming from the left side
 */
function handleThrowableImpact(
  socket: VTubeStudioWebSocket,
  appData: AppData,
  modelParameters: ModelParameters,
  config: ItemWithSoundIds,
  impactAudio: LoadedSoundData | null,
  angle: number,
  leftSide: boolean,
) {
  // Play the impact sound
  if (impactAudio !== null) {
    try {
      impactAudio.sound.volume =
        appData.sounds_config.global_volume * impactAudio.config.volume;

      impactAudio.sound.play();
    } catch (err) {
      console.error("failed to play audio", err);
    }
  }

  const { image } = config.config;

  // Make the VTuber model flinch from the impact
  flinch(socket, modelParameters, {
    angle,
    eyeState: appData.model_config.eyes_on_hit,
    magnitude: image.weight,
    leftSide,
    returnSpeed: 0.3,
  });

  // TODO: IMPACT DECAL
}

function createRootContainer(modelPosition: ModelPosition) {
  const elm = document.createElement("div");
  elm.classList.add("thrown");

  const style = elm.style;

  const originXPercent = ((modelPosition.positionX + 1) / 2) * 100;
  const originYPercent = (1 - (modelPosition.positionY + 1) / 2) * 100;

  style.width = "100%";
  style.height = "100%";
  style.transformOrigin = `${originXPercent}% ${originYPercent}%`;

  return elm;
}

/**
 * Creates a throwable image element, applies rotation and other
 * image styling
 *
 * @param config Configuration for the image
 * @param image The underlying image element itself
 * @param scaledWidth The scaled width of the image
 * @param scaledHeight The scaled height of the image
 * @param angle The angle of the image
 * @param spinSpeed Speed the image should spin at
 * @returns The image element
 */
function createThrownImage(
  config: ItemImageConfig,
  image: HTMLImageElement,

  scaledWidth: number,
  scaledHeight: number,

  angle: number,
  spinSpeed: MinMax,
): HTMLImageElement {
  const elm = image.cloneNode(true) as HTMLImageElement;
  elm.classList.add("animated");
  const style = elm.style;

  style.width = `${scaledWidth}px`;
  style.height = `${scaledHeight}px`;
  style.imageRendering = config.pixelate ? "pixelated" : "auto";

  // Spin speed is zero, should immediately spin all the way
  if (spinSpeed.max - spinSpeed.min === 0) {
    style.transform = "rotate(" + -angle + "deg)";
    return elm;
  }

  const clockwise = randomBool();
  const animationDuration = 3 / randomRange(spinSpeed.min, spinSpeed.max);

  style.animationName = clockwise ? "spinClockwise" : "spinCounterClockwise";
  style.animationDuration = `${animationDuration}s`;
  style.animationIterationCount = "infinite";

  // TODO: SLOW DOWN NEAR END? 1  / randomRange(spinSpeed.min, spinSpeed.max); AFTER data.throwDuration * 500 + data.delay

  return elm;
}

/**
 * Creates a container for handling the pivoting of a throwable
 *
 * @param scaledWidth Scaled width of the image
 * @param scaledHeight Scaled height of the image
 * @param modelPosition The position of the model
 * @param modelData Model and calibration data for the model
 * @param modelScale Scale of the model
 * @param angle Angle of the throwable
 * @returns The container element
 */
function createPivotContainer(
  scaledWidth: number,
  scaledHeight: number,
  modelPosition: ModelPosition,
  modelData: ModelCalibration,
  modelScale: number,
  angle: number,
) {
  const elm = document.createElement("div");
  elm.classList.add("thrown");

  const style = elm.style;

  const offsetX = percentRange(modelScale, modelData.x.min, modelData.x.max);
  const offsetY = percentRange(modelScale, modelData.y.min, modelData.y.max);

  const xPos = (modelPosition.positionX - offsetX + 1) / 2;
  const yPos = 1 - (modelPosition.positionY - offsetY + 1) / 2;

  // Random offsets to the X and Y positions
  const randX = (Math.random() * 100 - 50) * modelScale;
  const randY = (Math.random() * 100 - 50) * modelScale;

  const left = window.innerWidth * xPos - scaledWidth / 2 + randX;
  const top = window.innerHeight * yPos - scaledHeight / 2 + randY;

  style.left = `${left}px`;
  style.top = `${top}px`;
  style.transform = "rotate(" + angle + "deg)";

  return elm;
}

/**
 * Creates the container in charge of the movement for
 * a throwable item
 *
 * @param leftSide Whether the movement is coming from the left side
 * @param duration The duration of the whole animation
 * @param delayMs Delay before the movement begins
 * @returns The container element
 */
function createMovementContainer(
  leftSide: boolean,
  duration: number,
  delayMs: number,
) {
  const elm = document.createElement("div");
  elm.classList.add("animated");

  const style = elm.style;

  style.animationName = leftSide ? "throwLeft" : "throwRight";
  style.animationDuration = `${duration}ms`;
  style.animationDelay = `${delayMs}ms`;

  return elm;
}
