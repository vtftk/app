import { z } from "zod";

// When working with existing configs we allow the file to be a
// string to account for already uploaded file URLs
export const throwableImageSchema = z.union(
  [
    z.instanceof(File, {
      error: "Image file is required",
    }),
    z.string(),
  ],
  {
    error: "Image is required",
  },
);

export type ThrowableImageSchema = z.infer<typeof throwableImageSchema>;

export const itemImageConfigSchema = z.object({
  image: throwableImageSchema,
  scale: z.number(),
  weight: z.number(),
  pixelate: z.boolean(),
});

export const itemWindupConfigSchema = z.object({
  enabled: z.boolean(),
  duration: z.number(),
});

export const itemConfigSchema = z.object({
  image: itemImageConfigSchema,
  windup: itemWindupConfigSchema,
});

export const itemSchema = z.object({
  name: z.string().min(1, "You must specify a name"),
  config: itemConfigSchema,
  impactSoundIds: z.array(z.string()),
  windupSoundIds: z.array(z.string()),
});

export type ItemSchema = z.infer<typeof itemSchema>;
