import { z } from "zod"
import { FormFieldType } from "./types"

export function getFieldLabel(name: string, label?: string) {
  if (label) {
    return label
  }

  return name
    .split("_")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}

export function getFieldType(type: z.ZodType): FormFieldType {
  const typeName = (type as any)?._def?.typeName

  if (type instanceof z.ZodString || typeName === "ZodString") {
    return "text"
  }

  if (type instanceof z.ZodNumber || typeName === "ZodNumber") {
    return "number"
  }

  if (type instanceof z.ZodBoolean || typeName === "ZodBoolean") {
    return "boolean"
  }

  if (type instanceof z.ZodNullable || typeName === "ZodNullable") {
    const innerType = (type as any).unwrap?.() || (type as any)._def?.innerType
    return innerType ? getFieldType(innerType) : "text"
  }

  if (type instanceof z.ZodOptional || typeName === "ZodOptional") {
    const innerType = (type as any).unwrap?.() || (type as any)._def?.innerType
    return innerType ? getFieldType(innerType) : "text"
  }

  if (typeName === "ZodPipeline" || typeName === "ZodEffects") {
    const innerType = (type as any)._def?.in || (type as any)._def?.schema
    return innerType ? getFieldType(innerType) : "text"
  }

  return "unsupported"
}

export function getIsFieldOptional(type: z.ZodType) {
  const typeName = (type as any)?._def?.typeName
  return (
    type instanceof z.ZodOptional ||
    type instanceof z.ZodNull ||
    type instanceof z.ZodUndefined ||
    type instanceof z.ZodNullable ||
    typeName === "ZodOptional" ||
    typeName === "ZodNull" ||
    typeName === "ZodUndefined" ||
    typeName === "ZodNullable"
  )
}
