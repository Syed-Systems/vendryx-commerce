import styles from "../../styles/ui.module.css"

/** A 1px rule. MASTER.md §1: depth is a border, never a shadow. */
const Divider = ({ className }: { className?: string }) => (
  <hr className={`${styles.divider} ${className ?? ""}`} />
)

export default Divider
