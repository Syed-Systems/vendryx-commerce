import { zodResolver } from "@hookform/resolvers/zod"
import { FieldValues, Resolver, useForm, UseFormProps } from "react-hook-form"
import { z, ZodObject } from "zod"

import { ConfigField } from "../types"

interface UseExtendableFormProps<
  TSchema extends ZodObject<any>,
  TContext = any,
  TData = any,
  TTransformedValues extends FieldValues = z.infer<TSchema>
> extends Omit<
    UseFormProps<z.infer<TSchema>, TContext, TTransformedValues>,
    "resolver"
  > {
  schema: TSchema | any
  configs: ConfigField[]
  data?: TData
}

function createAdditionalDataSchema(configs: ConfigField[]) {
  return configs.reduce((acc, config) => {
    acc[config.name] = config.validation
    return acc
  }, {} as Record<string, z.ZodTypeAny>)
}

/**
 * Extracts the shape from a schema, handling ZodObject, ZodEffects, and ZodPipeline.
 */
function getSchemaShape(
  schema: any
): z.ZodRawShape {
  if (!schema) {
    return {}
  }

  // Handle direct ZodObject
  if (schema instanceof ZodObject || schema._def?.typeName === "ZodObject") {
    return schema.shape
  }

  // Handle pipeline / effects (.transform, .refine)
  if (
    schema._def?.typeName === "ZodPipeline" ||
    schema._def?.typeName === "ZodEffects"
  ) {
    const innerSchema = schema._def.in || schema._def.schema
    if (innerSchema) {
      return getSchemaShape(innerSchema)
    }
  }

  if (schema.shape) {
    return schema.shape
  }

  return {}
}

function createExtendedSchema(
  baseSchema: any,
  additionalDataSchema: Record<string, z.ZodTypeAny>
) {
  const hasAdditionalData = Object.keys(additionalDataSchema).length > 0
  if (!hasAdditionalData) {
    return baseSchema
  }

  const baseShape = getSchemaShape(baseSchema)

  const extendedObjectSchema = z.object({
    ...baseShape,
    additional_data: z.object(additionalDataSchema).optional(),
  })

  if (typeof baseSchema?.and === "function") {
    return baseSchema
      .superRefine((data: any, ctx: any) => {
        const result = extendedObjectSchema.safeParse(data)
        if (!result.success) {
          result.error.issues.forEach((issue) => ctx.addIssue({ ...issue }))
        }
      })
      .and(extendedObjectSchema)
  }

  return z.intersection(baseSchema, extendedObjectSchema)
}

function createExtendedDefaultValues<TData>(
  baseDefaultValues: any,
  configs: ConfigField[],
  data?: TData
) {
  const additional_data = configs.reduce((acc, config) => {
    const { name, defaultValue } = config

    acc[name] =
      typeof defaultValue === "function" ? defaultValue(data) : defaultValue
    return acc
  }, {} as Record<string, any>)

  return Object.assign(baseDefaultValues, { additional_data })
}

export const useExtendableForm = <
  TSchema extends ZodObject<any>,
  TContext = any,
  TTransformedValues extends FieldValues = z.infer<TSchema>
>({
  defaultValues: baseDefaultValues,
  schema: baseSchema,
  configs,
  data,
  ...props
}: UseExtendableFormProps<TSchema, TContext, any, TTransformedValues>) => {
  const additionalDataSchema = createAdditionalDataSchema(configs)
  const schema = createExtendedSchema(baseSchema, additionalDataSchema)
  const defaultValues = createExtendedDefaultValues(
    baseDefaultValues,
    configs,
    data
  )

  return useForm<z.infer<TSchema>, TContext, TTransformedValues>({
    ...props,
    defaultValues,
    // `schema` is assembled at runtime from the base schema plus the extension
    // fields, so its inferred type can't be related back to `TSchema` here.
    resolver: zodResolver(schema) as unknown as Resolver<
      z.infer<TSchema>,
      TContext,
      TTransformedValues
    >,
  })
}
