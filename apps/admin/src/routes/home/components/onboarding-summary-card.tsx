import {
  ArrowRight,
  CheckCircleSolid,
  CircleMiniSolid,
  Sparkles,
  XMark,
} from "@medusajs/icons"
import { Badge, Button, Container, Heading, Text } from "@medusajs/ui"
import { Link } from "react-router-dom"
import {
  useDismissOnboarding,
  type OnboardingStatus,
} from "../../../hooks/api/onboarding"

// ─────────────────────────────────────────────────────────────────────────────
// Store Setup Roadmap
// ─────────────────────────────────────────────────────────────────────────────
//
// Shows a clear, confidence-building launch checklist when store setup is in progress.
// Transforms the empty space into a purposeful action guide for the merchant.

export const OnboardingSummaryCard = ({
  onboarding,
  isLoading,
}: {
  onboarding: OnboardingStatus | undefined | null
  isLoading: boolean
}) => {
  const { mutate: dismissOnboarding } = useDismissOnboarding()

  if (isLoading && !onboarding) return null
  if (!onboarding) return null
  if (onboarding.is_completed || onboarding.dismissed) return null

  const percentage = Math.min(Math.round(onboarding.completion_percentage || 0), 100)
  const completedCount = onboarding.completed_steps_count
  const totalCount = onboarding.total_steps_count

  return (
    <Container className="flex flex-col p-0 overflow-hidden border border-ui-border-base rounded-xl bg-ui-bg-base shadow-sm">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-ui-border-base bg-ui-bg-subtle/40">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-blue-500/10 border border-blue-500/20 text-blue-600">
            <Sparkles className="w-4 h-4" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <Heading level="h3" className="text-sm font-semibold text-ui-fg-base">
                Store Setup Roadmap
              </Heading>
              <Badge color="blue" size="2xsmall">
                {percentage}% ready
              </Badge>
            </div>
            <Text size="xsmall" className="text-ui-fg-subtle">
              {completedCount} of {totalCount} setup steps completed
            </Text>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="transparent"
            size="small"
            onClick={() => dismissOnboarding()}
            className="text-ui-fg-muted hover:text-ui-fg-base text-xs"
            title="Dismiss checklist from overview"
          >
            <XMark className="w-3.5 h-3.5 mr-1" />
            Dismiss
          </Button>
          <Button variant="secondary" size="small" asChild className="text-xs">
            <Link to="/onboarding">
              Full Wizard
              <ArrowRight className="w-3.5 h-3.5 ml-1" />
            </Link>
          </Button>
        </div>
      </div>

      {/* Progress Bar */}
      <div className="w-full bg-ui-bg-component h-1.5">
        <div
          className="bg-blue-600 h-1.5 transition-all duration-500 ease-out"
          style={{ width: `${percentage}%` }}
        />
      </div>

      {/* Step List */}
      <div className="p-4 sm:p-5 flex flex-col gap-2">
        {onboarding.steps && onboarding.steps.length > 0 ? (
          onboarding.steps.map((step, index) => {
            const isCompleted = step.completed
            const isNext = !isCompleted && onboarding.steps.slice(0, index).every((s) => s.completed)

            return (
              <div
                key={step.id || index}
                className={`flex items-center justify-between p-3 rounded-lg transition-colors border ${
                  isNext
                    ? "bg-blue-500/5 border-blue-500/30 shadow-xs"
                    : isCompleted
                    ? "bg-ui-bg-subtle/30 border-transparent text-ui-fg-muted"
                    : "border-transparent opacity-75"
                }`}
              >
                <div className="flex items-center gap-3 min-w-0">
                  {isCompleted ? (
                    <CheckCircleSolid className="w-4 h-4 text-emerald-600 shrink-0" />
                  ) : isNext ? (
                    <div className="w-4 h-4 rounded-full border-2 border-blue-600 flex items-center justify-center shrink-0">
                      <div className="w-1.5 h-1.5 rounded-full bg-blue-600 animate-ping" />
                    </div>
                  ) : (
                    <CircleMiniSolid className="w-4 h-4 text-ui-fg-muted shrink-0" />
                  )}

                  <div className="flex flex-col min-w-0">
                    <Text
                      size="small"
                      weight={isNext ? "plus" : "regular"}
                      className={`truncate ${
                        isCompleted
                          ? "text-ui-fg-subtle line-through"
                          : isNext
                          ? "text-ui-fg-base font-medium"
                          : "text-ui-fg-muted"
                      }`}
                    >
                      {step.title}
                    </Text>
                    {step.description && (
                      <Text size="xsmall" className="text-ui-fg-subtle truncate max-w-md">
                        {step.description}
                      </Text>
                    )}
                  </div>
                </div>

                <div className="shrink-0 ml-3">
                  {isCompleted ? (
                    <span className="text-[11px] font-medium text-emerald-600 px-2 py-0.5 rounded-full bg-emerald-500/10">
                      Done
                    </span>
                  ) : (
                    <Button variant={isNext ? "primary" : "secondary"} size="small" asChild className="text-xs">
                      <Link to={step.href}>
                        {isNext ? "Configure" : "Start"}
                        <ArrowRight className="w-3 h-3 ml-1" />
                      </Link>
                    </Button>
                  )}
                </div>
              </div>
            )
          })
        ) : null}
      </div>
    </Container>
  )
}
