import { flinch } from "./flinch";
import playSound from "./playSound";
import { VTubeStudioWebSocket } from "./socket";
import { sleep, LoadedSoundData } from "../utils/async";
import { ModelPosition, ModelParameters } from "./model";
import { randomBool, randomRange, percentRange } from "../utils/math";
import { LARGEST_MODEL_SIZE, TOTAL_MODEL_SIZE_RANGE } from "../constants";
import {
  PhysicsEngine,
  PhysicsEngineConfig,
  createPhysicsEngine,
} from "./physics";
import {
  ItemConfig,
  OverlayConfig,
  PhysicsConfig,
  ThrowDirection,
  ItemImageConfig,
  ModelCalibration,
  ThrowablesConfig,
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

export async function throwItem(
  // Socket for sending messages to vtube studio
  socket: VTubeStudioWebSocket,

  // Configuration
  itemConfig: ItemConfig,
  overlayConfig: OverlayConfig,

  // Image
  image: HTMLImageElement,

  // Sounds
  impactAudio: LoadedSoundData | null,
  windupAudio: LoadedSoundData | null,

  // Model data
  modelParameters: ModelParameters,
  modelPosition: ModelPosition,
  modelData: ModelCalibration,
) {
  // Model is not available
  if (!modelPosition) return;

  const { throwables_config, physics_config } = overlayConfig;

  // Determine scale of the model relative to the calibrated minimum and maximum sizes
  const modelScale =
    (modelPosition.size + LARGEST_MODEL_SIZE) / TOTAL_MODEL_SIZE_RANGE;

  const leftSide: boolean = isRandomDirectionLeft(
    throwables_config.direction,
    percentRange(modelScale, modelData.x.min, modelData.x.max),
  );

  const angle =
    randomRange(
      throwables_config.throw_angle.min,
      throwables_config.throw_angle.max,
    ) *
    // Flip angle when coming from the right side
    (leftSide ? 1 : -1);

  const { windup } = itemConfig;

  if (windup.enabled && windupAudio !== null) {
    // Play the windup sound
    playSound(windupAudio, overlayConfig.sounds_config);

    await sleep(windup.duration);
  }

  const thrown = createThrownImage(
    itemConfig.image,
    image,
    modelScale,
    angle,
    throwables_config,
  );
  const movement = createMovementContainer(thrown, leftSide, throwables_config);
  const pivot = createPivotContainer(
    movement,
    modelPosition,
    modelData,
    modelScale,
    angle,
  );
  const root = createRootContainer(pivot, modelPosition);

  document.body.appendChild(root);

  // Impact is encountered half way through the animation
  const impactTimeout =
    throwables_config.duration / 2 + throwables_config.impact_delay;

  // Wait for the impact to occur
  await sleep(impactTimeout);

  // Handle point of impact
  handleThrowableImpact(
    socket,
    overlayConfig,
    modelParameters,
    itemConfig,
    impactAudio,
    angle,
    leftSide,
  );

  // No physics to apply
  if (!physics_config.enabled) {
    // Wait remaining duration before removing
    await sleep(throwables_config.duration / 2);
    // Remove after complete
    document.body.removeChild(root);
    return;
  }

  // Strip animations and transforms before applying physics
  movement.style.animationName = "";
  pivot.style.transform = "";
  thrown.style.transform = "";

  // Convert the item into a physics object
  throwItemPhysics(physics_config, movement, root, leftSide, angle);
}

function throwItemPhysics(
  physics_config: PhysicsConfig,
  movement: HTMLDivElement,
  root: HTMLDivElement,
  leftSide: boolean,
  angle: number,
) {
  // Initialize the physics engine
  if (physicsEngine === null) {
    const { fps, gravity_multiplier } = physics_config;

    physicsEngine = createPhysicsEngine({
      fps: fps,
      gravityMultiplier: gravity_multiplier,
    });
  }

  const { horizontal_multiplier, vertical_multiplier } = physics_config;

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
 * @param overlayConfig Global app data settings
 * @param modelParameters Parameters for the current model
 * @param itemConfig Configuration for the thrown item
 * @param impactAudio Audio element to play when the item impacts the target
 * @param angle Angle the item was thrown at
 * @param leftSide Whether the item is coming from the left side
 */
function handleThrowableImpact(
  socket: VTubeStudioWebSocket,
  overlayConfig: OverlayConfig,
  modelParameters: ModelParameters,
  itemConfig: ItemConfig,
  impactAudio: LoadedSoundData | null,
  angle: number,
  leftSide: boolean,
) {
  // Play the impact sound
  if (impactAudio !== null) {
    playSound(impactAudio, overlayConfig.sounds_config);
  }

  const { image } = itemConfig;

  // Make the VTuber model flinch from the impact
  flinch(socket, modelParameters, {
    angle,
    eyeState: overlayConfig.model_config.eyes_on_hit,
    magnitude: image.weight,
    leftSide,
    returnSpeed: 0.3,
  });

  // TODO: IMPACT DECAL
}

function createThrownImage(
  imageConfig: ItemImageConfig,
  image: HTMLImageElement,
  modelScale: number,
  angle: number,
  throwables_config: ThrowablesConfig,
): HTMLImageElement {
  const { item_scale, spin_speed } = throwables_config;
  const itemScale = percentRange(modelScale, item_scale.min, item_scale.max);

  const scaledWidth = image.width * imageConfig.scale * itemScale;
  const scaledHeight = image.height * imageConfig.scale * itemScale;

  const elm = image.cloneNode(true) as HTMLImageElement;
  elm.classList.add("animated");
  const style = elm.style;

  style.width = `${scaledWidth}px`;
  style.height = `${scaledHeight}px`;
  style.imageRendering = imageConfig.pixelate ? "pixelated" : "auto";

  // Spin speed is zero, should immediately spin all the way
  if (spin_speed.max - spin_speed.min === 0) {
    style.transform = "rotate(" + -angle + "deg)";
    return elm;
  }

  const clockwise = randomBool();
  const animationDuration = 3 / randomRange(spin_speed.min, spin_speed.max);

  style.animationName = clockwise ? "spinClockwise" : "spinCounterClockwise";
  style.animationDuration = `${animationDuration}s`;
  style.animationIterationCount = "infinite";

  // TODO: SLOW DOWN NEAR END? 1  / randomRange(spinSpeed.min, spinSpeed.max); AFTER data.throwDuration * 500 + data.delay

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
  thrown: HTMLImageElement,
  leftSide: boolean,
  throwables_config: ThrowablesConfig,
) {
  const { duration, impact_delay } = throwables_config;

  const elm = document.createElement("div");
  elm.classList.add("animated");

  const style = elm.style;

  style.animationName = leftSide ? "throwLeft" : "throwRight";
  style.animationDuration = `${duration}ms`;
  style.animationDelay = `${impact_delay}ms`;

  elm.appendChild(thrown);

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
  movement: HTMLDivElement,
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

  const left = window.innerWidth * xPos - movement.clientWidth / 2 + randX;
  const top = window.innerHeight * yPos - movement.clientHeight / 2 + randY;

  style.left = `${left}px`;
  style.top = `${top}px`;
  style.transform = "rotate(" + angle + "deg)";

  elm.appendChild(movement);

  return elm;
}

function createRootContainer(
  pivot: HTMLDivElement,
  modelPosition: ModelPosition,
) {
  const elm = document.createElement("div");
  elm.classList.add("t-root");

  const style = elm.style;

  const originXPercent = ((modelPosition.positionX + 1) / 2) * 100;
  const originYPercent = (1 - (modelPosition.positionY + 1) / 2) * 100;

  style.transformOrigin = `${originXPercent}% ${originYPercent}%`;

  elm.appendChild(pivot);

  return elm;
}
