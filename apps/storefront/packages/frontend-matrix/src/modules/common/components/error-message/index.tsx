import styles from "../../styles/ui.module.css"

/**
 * Matrix error treatment: mono, red, and marked with a 2px bar rather than an
 * icon — the source design system has no icon set for state, only rules.
 */
const ErrorMessage = ({
  error,
  "data-testid": dataTestid,
}: {
  error?: string | null
  "data-testid"?: string
}) => {
  if (!error) {
    return null
  }

  return (
    <div className={styles.error} role="alert" data-testid={dataTestid}>
      <span className={styles.errorBar} aria-hidden />
      <span>{error}</span>
    </div>
  )
}

export default ErrorMessage
