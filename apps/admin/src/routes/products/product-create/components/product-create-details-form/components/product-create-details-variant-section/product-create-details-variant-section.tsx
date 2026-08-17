import {
  Alert,
  Button,
  Checkbox,
  clx,
  Heading,
  Hint,
  InlineTip,
  Input,
  Label,
  Text,
} from "@medusajs/ui"
import { PlusMini, Trash } from "@medusajs/icons"
import {
  FieldArrayWithId,
  useFieldArray,
  UseFormReturn,
  useWatch,
} from "react-hook-form"
import { useTranslation } from "react-i18next"
import { useEffect, useMemo, useState } from "react"

import { Form } from "../../../../../../../components/common/form"
import { SortableList } from "../../../../../../../components/common/sortable-list"
import { SwitchBox } from "../../../../../../../components/common/switch-box"
import { ChipInput } from "../../../../../../../components/inputs/chip-input"
import { Combobox } from "../../../../../../../components/inputs/combobox"
import { ProductCreateSchemaType } from "../../../../types"
import { decorateVariantsWithDefaultValues } from "../../../../utils"
import { productOptionsQueryKeys } from "../../../../../../../hooks/api"
import { useComboboxData } from "../../../../../../../hooks/use-combobox-data"
import { sdk } from "../../../../../../../lib/client"
import { AdminProductOption, HttpTypes } from "@medusajs/types"

type ProductCreateVariantsSectionProps = {
  form: UseFormReturn<ProductCreateSchemaType>
}

type ProductOptionFormValue = {
  title: string
  values: string[]
  id?: string
  value_ids?: string[]
}

type SelectedOptionInput =
  | Pick<AdminProductOption, "id" | "title" | "values">
  | { title: string; values: string[] }

const getPermutations = (
  data: { title: string; values: string[] }[]
): { [key: string]: string }[] => {
  if (data.length === 0) {
    return []
  }

  if (data.length === 1) {
    return data[0].values.map((value) => ({ [data[0].title]: value }))
  }

  const toProcess = data[0]
  const rest = data.slice(1)

  return toProcess.values.flatMap((value) => {
    return getPermutations(rest).map((permutation) => {
      return {
        [toProcess.title]: value,
        ...permutation,
      }
    })
  })
}

const getVariantName = (options: Record<string, string>) => {
  return Object.values(options).join(" / ")
}

export const ProductCreateVariantsSection = ({
  form,
}: ProductCreateVariantsSectionProps) => {
  const { t } = useTranslation()
  const [newOptionName, setNewOptionName] = useState("")

  const variants = useFieldArray({
    control: form.control,
    name: "variants",
  })

  const watchedAreVariantsEnabled = useWatch({
    control: form.control,
    name: "enable_variants",
    defaultValue: false,
  })

  const watchedOptions = useWatch({
    control: form.control,
    name: "options",
    defaultValue: [],
  })

  const watchedVariants = useWatch({
    control: form.control,
    name: "variants",
    defaultValue: [],
  })

  const showInvalidOptionsMessage = !!form.formState.errors.options?.length
  const showInvalidVariantsMessage =
    form.formState.errors.variants?.root?.message === "invalid_length"

  const selectedExistingOptionIds = useMemo(
    () =>
      watchedOptions.map((opt) => opt.id).filter((id): id is string => !!id),
    [watchedOptions]
  )

  const productOptionsCombobox = useComboboxData({
    queryKey: productOptionsQueryKeys.list({ is_exclusive: false }),
    queryFn: (params) =>
      sdk.admin.productOption.list({
        ...params,
        is_exclusive: false,
        fields: "id,title,values.id,values.value,values.rank",
      } as HttpTypes.AdminProductOptionListParams),
    getOptions: (data) =>
      data.product_options.map((option) => ({
        label: option.title,
        value: option.id,
        option,
      })),
  })

  const [optionDetailsById, setOptionDetailsById] = useState(
    () => new Map<string, AdminProductOption>()
  )

  useEffect(() => {
    setOptionDetailsById((prev) => {
      let next: Map<string, AdminProductOption> | undefined

      for (const choice of productOptionsCombobox.options) {
        if (choice.option && prev.get(choice.value) !== choice.option) {
          next ??= new Map(prev)
          next.set(choice.value, choice.option)
        }
      }

      return next ?? prev
    })
  }, [productOptionsCombobox.options])

  const knownOptionIds = useMemo(() => {
    const ids = new Set<string>(selectedExistingOptionIds)
    productOptionsCombobox.options.forEach((option) => ids.add(option.value))
    return ids
  }, [selectedExistingOptionIds, productOptionsCombobox.options])

  const productOptionChoices = useMemo(() => {
    const merged = new Map<string, { label: string; value: string }>()
    productOptionsCombobox.options.forEach((option) =>
      merged.set(option.value, option)
    )

    watchedOptions.forEach((opt) => {
      const value = opt.id || opt.title
      if (value && !merged.has(value)) {
        merged.set(value, { value, label: opt.title || value })
      }
    })

    return [...merged.values()]
  }, [productOptionsCombobox.options, watchedOptions])

  const selectedOptionValues = useMemo(() => {
    return watchedOptions
      .filter((opt) => !!(opt.id || opt.title)?.trim())
      .map((opt) => opt.id || opt.title)
  }, [watchedOptions])

  const handleProductOptionSelect = (optionValues: string[]) => {
    const cleanValues = (optionValues || []).filter((val) => !!val?.trim())
    const currentWatchedOptions = form
      .getValues("options")
      .filter((opt) => !!(opt.id || opt.title)?.trim())

    const updatedOptions: ProductOptionFormValue[] = []

    cleanValues.forEach((val) => {
      if (knownOptionIds.has(val)) {
        const details = optionDetailsById.get(val)
        if (details) {
          const current = currentWatchedOptions.find((opt) => opt.id === val)
          updatedOptions.push({
            id: details.id,
            title: details.title,
            values:
              current?.values ||
              details.values?.map((v) => v.value) ||
              [],
            value_ids:
              current?.value_ids ||
              details.values?.map((v) => v.id) ||
              [],
          })
          return
        }
      }

      const current = currentWatchedOptions.find(
        (opt) => !opt.id && opt.title.toLowerCase() === val.toLowerCase()
      )
      updatedOptions.push({
        title: val,
        values: current?.values || [],
      })
    })

    form.setValue("options", updatedOptions)
    generateAndSetVariants(updatedOptions)
  }

  const generateAndSetVariants = (options: ProductOptionFormValue[]) => {
    const validOptions = options.filter(
      ({ values }) => values && values.length > 0
    )
    if (validOptions.length === 0) {
      return
    }

    const permutations = getPermutations(validOptions)

    const newVariants = permutations.map((permutation, index) => ({
      title: getVariantName(permutation),
      options: permutation,
      should_create: true,
      variant_rank: index,
      inventory: [{ inventory_item_id: "", required_quantity: "" }],
    }))

    form.setValue("variants", newVariants)
  }

  const handleAddNewOption = (customTitle?: string) => {
    const title = (customTitle || newOptionName).trim()
    if (!title) return

    const currentOptions = form
      .getValues("options")
      .filter((opt) => !!(opt.id || opt.title)?.trim())

    if (
      currentOptions.some(
        (opt) => opt.title.toLowerCase() === title.toLowerCase()
      )
    ) {
      setNewOptionName("")
      return
    }

    const updated = [
      ...currentOptions,
      {
        title,
        values: [],
      },
    ]
    form.setValue("options", updated)
    setNewOptionName("")
  }

  const handleAddCustomVariant = () => {
    const currentOptions = form
      .getValues("options")
      .filter((opt) => !!(opt.id || opt.title)?.trim())

    let optionsMap: Record<string, string> = {}

    if (currentOptions.length === 0) {
      const defaultOpt = { title: "Option", values: ["Standard"] }
      form.setValue("options", [defaultOpt])
      optionsMap = { Option: "Standard" }
    } else {
      currentOptions.forEach((opt) => {
        const val = opt.values?.[0] || "Default"
        optionsMap[opt.title] = val
        if (!opt.values || opt.values.length === 0) {
          opt.values = [val]
        }
      })
      form.setValue("options", [...currentOptions])
    }

    const currentVariants = form.getValues("variants") || []
    const variantTitle =
      getVariantName(optionsMap) || `Variant ${currentVariants.length + 1}`

    const newVariant = {
      title: variantTitle,
      options: optionsMap,
      should_create: true,
      variant_rank: currentVariants.length,
      inventory: [{ inventory_item_id: "", required_quantity: "" }],
    }

    form.setValue("variants", [...currentVariants, newVariant])
  }

  const handleValueChange = (optionId: string, valueIds: string[]) => {
    if (valueIds.length === 0) {
      return
    }

    const currentOption = watchedOptions.find((opt) => opt.id === optionId)
    if (!currentOption) {
      return
    }

    const productOption = optionDetailsById.get(optionId)
    const existingValueIds = new Set(
      productOption?.values?.map((v) => v.id) || []
    )

    const validValueIds: string[] = []
    const newValueNames: string[] = []

    valueIds.forEach((id) => {
      if (existingValueIds.has(id)) {
        validValueIds.push(id)
      } else {
        newValueNames.push(id)
      }
    })

    const updatedOptions = watchedOptions.map((opt) => {
      if (opt.id === optionId) {
        const selectedExistingValues =
          productOption?.values
            ?.filter((v) => validValueIds.includes(v.id))
            .map((v) => v.value) || []

        return {
          ...opt,
          value_ids: valueIds,
          values: [...selectedExistingValues, ...newValueNames],
        }
      }
      return opt
    })

    form.setValue("options", updatedOptions)
    generateAndSetVariants(updatedOptions)
  }

  const handleNewOptionValueChange = (
    optionTitle: string,
    valueNames: string[]
  ) => {
    const updatedOptions = watchedOptions.map((opt) => {
      if (!opt.id && opt.title === optionTitle) {
        return {
          ...opt,
          values: valueNames,
        }
      }
      return opt
    })

    form.setValue("options", updatedOptions)
    generateAndSetVariants(updatedOptions)
  }

  const handleRankChange = (
    items: FieldArrayWithId<ProductCreateSchemaType, "variants">[]
  ) => {
    const update = items.map((item, index) => {
      const variant = watchedVariants.find((v) => v.title === item.title)

      return {
        id: item.id,
        ...(variant || item),
        variant_rank: index,
      }
    })

    variants.replace(update)
  }

  const getCheckboxState = (variants: ProductCreateSchemaType["variants"]) => {
    if (variants.length === 0) return false
    if (variants.every((variant) => variant.should_create)) {
      return true
    }

    if (variants.some((variant) => variant.should_create)) {
      return "indeterminate"
    }

    return false
  }

  const onCheckboxChange = (value: boolean | "indeterminate") => {
    switch (value) {
      case true: {
        const update = watchedVariants.map((variant) => {
          return {
            ...variant,
            should_create: true,
          }
        })

        form.setValue("variants", update)
        break
      }
      case false: {
        const update = watchedVariants.map((variant) => {
          return {
            ...variant,
            should_create: false,
          }
        })

        form.setValue("variants", decorateVariantsWithDefaultValues(update))
        break
      }
      case "indeterminate":
        break
    }
  }

  const createDefaultOptionAndVariant = () => {
    form.setValue("options", [
      {
        title: "Default option",
        values: ["Default option value"],
      },
    ])
    form.setValue(
      "variants",
      decorateVariantsWithDefaultValues([
        {
          title: "Default variant",
          should_create: true,
          variant_rank: 0,
          options: {
            "Default option": "Default option value",
          },
          inventory: [{ inventory_item_id: "", required_quantity: "" }],
          is_default: true,
        },
      ])
    )
  }

  return (
    <div id="variants" className="flex flex-col gap-y-8">
      <div className="flex flex-col gap-y-6">
        <Heading level="h2">{t("products.create.variants.header")}</Heading>
        <SwitchBox
          control={form.control}
          name="enable_variants"
          label={t("products.create.variants.subHeadingTitle")}
          description={t("products.create.variants.subHeadingDescription")}
          onCheckedChange={(checked) => {
            if (checked) {
              form.setValue("options", [])
              form.setValue("variants", [])
            } else {
              createDefaultOptionAndVariant()
            }
          }}
        />
      </div>
      {watchedAreVariantsEnabled && (
        <>
          <div className="flex flex-col gap-y-6">
            <div className="flex items-center justify-between">
              <div className="flex flex-col">
                <Label weight="plus">
                  {t("products.create.variants.productOptions.label")}
                </Label>
                <Hint>{t("products.create.variants.productOptions.hint")}</Hint>
              </div>
            </div>

            {showInvalidOptionsMessage && (
              <Alert dismissible variant="error">
                {t("products.create.errors.options")}
              </Alert>
            )}

            <div className="flex flex-col gap-y-3">
              <div className="flex items-center gap-x-2">
                <div className="flex-1">
                  <Combobox
                    value={selectedOptionValues}
                    onChange={(value) =>
                      handleProductOptionSelect(value as string[])
                    }
                    options={productOptionChoices}
                    searchValue={productOptionsCombobox.searchValue}
                    onSearchValueChange={
                      productOptionsCombobox.onSearchValueChange
                    }
                    fetchNextPage={productOptionsCombobox.fetchNextPage}
                    isFetchingNextPage={
                      productOptionsCombobox.isFetchingNextPage
                    }
                    shouldAlwaysShowCreateOption
                    onCreateOption={async (options: any) => {
                      const optionTitle = Array.isArray(options)
                        ? options[options.length - 1]?.trim()
                        : typeof options === "string"
                        ? options.trim()
                        : ""

                      if (optionTitle) {
                        handleAddNewOption(optionTitle)
                      }
                    }}
                    placeholder={t(
                      "products.fields.options.optionTitlePlaceholder"
                    )}
                    disabled={productOptionsCombobox.isLoading}
                    displayMode="chips"
                  />
                </div>
              </div>

              <div className="flex items-center gap-x-2">
                <Input
                  size="small"
                  placeholder="Or type new option name (e.g. Material, Fit)"
                  value={newOptionName}
                  onChange={(e) => setNewOptionName(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault()
                      handleAddNewOption()
                    }
                  }}
                />
                <Button
                  type="button"
                  variant="secondary"
                  size="small"
                  onClick={() => handleAddNewOption()}
                  disabled={!newOptionName.trim()}
                >
                  <PlusMini /> Add Option
                </Button>
              </div>
            </div>
          </div>

          {watchedOptions.filter((opt) => !!(opt.id || opt.title)?.trim())
            .length > 0 && (
            <div className="flex flex-col gap-y-4">
              <div className="flex flex-col">
                <Label weight="plus">{t("fields.values")}</Label>
                <Hint>
                  Add values for each option (type a value and press Enter)
                </Hint>
              </div>
              <div className="flex flex-col gap-y-3">
                {watchedOptions
                  .filter((opt) => !!(opt.id || opt.title)?.trim())
                  .map((opt, index) => {
                    if (opt.id) {
                      const productOption = optionDetailsById.get(opt.id)

                      const existingValues = productOption?.values || []
                      const customValueNames =
                        opt.values?.filter(
                          (v) => !existingValues.some((ev) => ev.value === v)
                        ) || []

                      const existingValueOptions = [...existingValues]
                        .sort((a, b) => {
                          const rankA = a.rank ?? Number.MAX_VALUE
                          const rankB = b.rank ?? Number.MAX_VALUE
                          return rankA - rankB
                        })
                        .map((v) => ({
                          value: v.id,
                          label: v.value,
                        }))

                      const customValueOptions = customValueNames.map((v) => ({
                        value: v,
                        label: v,
                      }))

                      const valueOptions = [
                        ...existingValueOptions,
                        ...customValueOptions,
                      ]

                      return (
                        <div key={opt.id} className="flex flex-col gap-y-2">
                          <div className="flex items-center justify-between">
                            <Label size="small" weight="plus">
                              {opt.title}
                            </Label>
                            <button
                              type="button"
                              onClick={() => {
                                const updated = form
                                  .getValues("options")
                                  .filter((o) => o.id !== opt.id)
                                form.setValue("options", updated)
                                generateAndSetVariants(updated)
                              }}
                              className="text-ui-fg-muted hover:text-ui-fg-subtle text-xs"
                            >
                              {t("actions.delete")}
                            </button>
                          </div>
                          <Combobox
                            value={opt.value_ids ?? []}
                            onChange={(value) =>
                              handleValueChange(opt.id!, value as string[])
                            }
                            options={valueOptions}
                            placeholder="Select or enter values (press Enter)"
                            displayMode="chips"
                          />
                        </div>
                      )
                    } else {
                      return (
                        <div key={index} className="flex flex-col gap-y-2">
                          <div className="flex items-center justify-between">
                            <Label size="small" weight="plus">
                              {opt.title}
                            </Label>
                            <button
                              type="button"
                              onClick={() => {
                                const updated = form
                                  .getValues("options")
                                  .filter((o) => o.title !== opt.title)
                                form.setValue("options", updated)
                                generateAndSetVariants(updated)
                              }}
                              className="text-ui-fg-muted hover:text-ui-fg-subtle text-xs"
                            >
                              {t("actions.delete")}
                            </button>
                          </div>
                          <ChipInput
                            value={opt.values ?? []}
                            onChange={(value) =>
                              handleNewOptionValueChange(opt.title, value)
                            }
                            placeholder="Type value and press Enter (e.g. S, M, L)"
                          />
                        </div>
                      )
                    }
                  })}
              </div>
            </div>
          )}

          <div className="grid grid-cols-1 gap-x-4 gap-y-8">
            <div className="flex flex-col gap-y-6">
              <div className="flex items-center justify-between">
                <div className="flex flex-col">
                  <Label weight="plus">
                    {t("products.create.variants.productVariants.label")}
                  </Label>
                  <Hint>
                    {t("products.create.variants.productVariants.hint")}
                  </Hint>
                </div>
                <Button
                  type="button"
                  variant="secondary"
                  size="small"
                  onClick={handleAddCustomVariant}
                >
                  <PlusMini /> Add Variant
                </Button>
              </div>

              {!showInvalidOptionsMessage && showInvalidVariantsMessage && (
                <Alert dismissible variant="error">
                  {t("products.create.errors.variants")}
                </Alert>
              )}

              {variants.fields.length > 0 ? (
                <div className="overflow-hidden rounded-xl border">
                  <div
                    className="bg-ui-bg-component text-ui-fg-subtle grid items-center gap-3 border-b px-6 py-2.5"
                    style={{
                      gridTemplateColumns: `20px 28px 1fr 32px`,
                    }}
                  >
                    <div>
                      <Checkbox
                        className="relative"
                        checked={getCheckboxState(watchedVariants)}
                        onCheckedChange={onCheckboxChange}
                      />
                    </div>
                    <div />
                    <div>
                      <Text size="small" leading="compact" weight="plus">
                        Variant Title & Options
                      </Text>
                    </div>
                    <div />
                  </div>
                  <SortableList
                    items={variants.fields}
                    onChange={handleRankChange}
                    renderItem={(item, index) => {
                      return (
                        <SortableList.Item
                          id={item.id}
                          className={clx("bg-ui-bg-base border-b", {
                            "border-b-0": index === variants.fields.length - 1,
                          })}
                        >
                          <div
                            className="text-ui-fg-subtle grid w-full items-center gap-3 px-6 py-2.5"
                            style={{
                              gridTemplateColumns: `20px 28px 1fr 32px`,
                            }}
                          >
                            <Form.Field
                              control={form.control}
                              name={`variants.${index}.should_create` as const}
                              render={({
                                field: { value, onChange, ...field },
                              }) => {
                                return (
                                  <Form.Item>
                                    <Form.Control>
                                      <Checkbox
                                        className="relative"
                                        {...field}
                                        checked={value}
                                        onCheckedChange={onChange}
                                      />
                                    </Form.Control>
                                  </Form.Item>
                                )
                              }}
                            />
                            <SortableList.DragHandle />
                            <div className="flex items-center gap-x-2">
                              <Text size="small" weight="plus" leading="compact">
                                {item.title || `Variant ${index + 1}`}
                              </Text>
                              {item.options &&
                                Object.entries(item.options).map(
                                  ([k, v], optIdx) => (
                                    <span
                                      key={optIdx}
                                      className="bg-ui-bg-subtle text-ui-fg-muted rounded px-1.5 py-0.5 text-xs"
                                    >
                                      {k}: {v}
                                    </span>
                                  )
                                )}
                            </div>
                            <button
                              type="button"
                              onClick={() => {
                                variants.remove(index)
                              }}
                              className="text-ui-fg-muted hover:text-ui-fg-error flex items-center justify-center p-1"
                              title="Delete variant"
                            >
                              <Trash className="h-4 w-4" />
                            </button>
                          </div>
                        </SortableList.Item>
                      )
                    }}
                  />
                </div>
              ) : (
                <div className="flex flex-col items-center justify-center rounded-xl border border-dashed p-8 text-center">
                  <Text size="small" className="text-ui-fg-subtle mb-3">
                    Add options with values above to generate variants, or click
                    below to create a variant directly.
                  </Text>
                  <Button
                    type="button"
                    variant="secondary"
                    size="small"
                    onClick={handleAddCustomVariant}
                  >
                    <PlusMini /> Create Variant
                  </Button>
                </div>
              )}

              {variants.fields.length > 0 && (
                <InlineTip label={t("general.tip")}>
                  {t("products.create.variants.productVariants.tip")}
                </InlineTip>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  )
}
