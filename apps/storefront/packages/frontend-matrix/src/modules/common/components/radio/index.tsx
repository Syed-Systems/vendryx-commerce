import styles from "../../styles/ui.module.css"

/**
 * The one circle in the theme. MASTER.md forbids radii, but a square radio
 * reads as a checkbox and lies about whether the choice is exclusive —
 * accessibility beats the style rule, per the ladder's standing exception.
 */
const Radio = ({
  checked,
  "data-testid": dataTestId,
}: {
  checked: boolean
  "data-testid"?: string
}) => (
  <span
    className={styles.radio}
    data-state={checked ? "checked" : "unchecked"}
    data-testid={dataTestId || "radio-button"}
    aria-hidden
  >
    {checked && <span className={styles.radioDot} />}
  </span>
)

export default Radio
