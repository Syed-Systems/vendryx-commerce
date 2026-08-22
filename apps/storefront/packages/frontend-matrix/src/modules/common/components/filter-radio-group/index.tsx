import { EllipseMiniSolid } from "@medusajs/icons"
import { Label, RadioGroup, Text, clx } from "@medusajs/ui"

type FilterRadioGroupProps = {
  title: string
  items: {
    value: string
    label: string
  }[]
  value: any
  handleChange: (...args: any[]) => void
  "data-testid"?: string
}

const FilterRadioGroup = ({
  title,
  items,
  value,
  handleChange,
  "data-testid": dataTestId,
}: FilterRadioGroupProps) => {
  return (
    <div className="flex flex-col gap-y-3">
      <Text className="txt-compact-small-plus text-ui-fg-muted">{title}</Text>
      <RadioGroup
        data-testid={dataTestId}
        onValueChange={handleChange}
        className="flex flex-col gap-y-3"
      >
        {items?.map((i) => (
          // Marker after the label, not before it.
          //
          // It used to lead, and the selected row carried `ml-[-23px]` so the dot could
          // hang outside — which moved the whole row, leaving the selected option 23px
          // left of every other one. Leading it without that offset instead indents every
          // label away from the container edge the heading, hero and product grid all sit
          // on. Trailing it keeps one left edge for the entire column and still marks the
          // selection with something other than colour.
          <div key={i.value} className="flex items-center gap-x-2">
            <RadioGroup.Item
              checked={i.value === value}
              className="hidden peer"
              id={i.value}
              value={i.value}
            />
            <Label
              htmlFor={i.value}
              className={clx(
                "!txt-compact-small !transform-none text-ui-fg-subtle hover:cursor-pointer",
                {
                  "text-ui-fg-base": i.value === value,
                }
              )}
              data-testid="radio-label"
              data-active={i.value === value}
            >
              {i.label}
            </Label>
            {i.value === value && <EllipseMiniSolid aria-hidden />}
          </div>
        ))}
      </RadioGroup>
    </div>
  )
}

export default FilterRadioGroup
