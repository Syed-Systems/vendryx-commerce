"use client"

import React, { useId } from "react"
import styles from "../../styles/ui.module.css"

type CheckboxProps = {
  checked?: boolean
  onChange?: () => void
  label: string
  name?: string
  "data-testid"?: string
}

/**
 * A square that fills gold. Replaces @medusajs/ui's Checkbox — which also fixed
 * `id="checkbox"` for every instance on the page, so two of them shared one id
 * and the label activated the wrong box.
 */
const CheckboxWithLabel: React.FC<CheckboxProps> = ({
  checked = true,
  onChange,
  label,
  name,
  "data-testid": dataTestId,
}) => {
  const id = useId()

  return (
    <div className={styles.checkRow}>
      <button
        type="button"
        role="checkbox"
        id={id}
        className={styles.checkbox}
        data-state={checked ? "checked" : "unchecked"}
        aria-checked={checked}
        onClick={onChange}
        name={name}
        data-testid={dataTestId}
      >
        {checked && (
          <svg width="11" height="11" viewBox="0 0 12 12" aria-hidden>
            <path
              d="M1.5 6.5 4.5 9.5 10.5 2.5"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="square"
            />
          </svg>
        )}
      </button>
      <label htmlFor={id} className={styles.checkLabel}>
        {label}
      </label>
    </div>
  )
}

export default CheckboxWithLabel
