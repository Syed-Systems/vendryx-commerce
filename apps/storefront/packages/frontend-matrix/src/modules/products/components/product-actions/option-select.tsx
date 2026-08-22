import { HttpTypes } from "@medusajs/types"
import { clx } from "@medusajs/ui"
import React from "react"

type OptionSelectProps = {
  option: HttpTypes.StoreProductOption
  current: string | undefined
  updateOption: (title: string, value: string) => void
  title: string
  disabled: boolean
  "data-testid"?: string
}

const OptionSelect: React.FC<OptionSelectProps> = ({
  option,
  current,
  updateOption,
  title,
  "data-testid": dataTestId,
  disabled,
}) => {
  const filteredOptions = (option.values ?? []).map((v) => v.value)

  return (
    <div className="flex flex-col gap-y-2.5">
      <div className="flex items-center justify-between text-xs">
        <span className="font-semibold uppercase tracking-wider text-[#f5f5f0]">
          {title}:{" "}
          <span className="font-normal text-[#a0a0a0]">
            {current || "Select option"}
          </span>
        </span>
        {title.toLowerCase().includes("size") && (
          <span className="text-[11px] text-[#a0a0a0] underline cursor-pointer hover:text-[#c9a96e] transition-colors">
            Size Guide
          </span>
        )}
      </div>

      <div className="flex flex-wrap gap-2" data-testid={dataTestId}>
        {filteredOptions.map((v) => {
          const isSelected = v === current
          return (
            <button
              onClick={() => updateOption(option.id, v)}
              key={v}
              className={clx(
                "min-w-[48px] px-3.5 py-2 text-xs font-medium rounded-none border transition-all duration-200 focus:outline-none",
                isSelected
                  ? "bg-zinc-950 text-white border-[#c9a96e] scale-100 font-semibold"
                  : "bg-[#141414] text-[#f5f5f0] border-white/10 hover:border-[#c9a96e] hover:bg-[#242424] active:scale-95"
              )}
              disabled={disabled}
              data-testid="option-button"
            >
              {v}
            </button>
          )
        })}
      </div>
    </div>
  )
}

export default OptionSelect
