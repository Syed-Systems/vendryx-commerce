import {
  ArrowRight,
  BuildingStorefront,
  CheckCircleSolid,
  CreditCard,
  ShoppingBag,
  Sparkles,
  TruckFast,
} from "@medusajs/icons"
import { Badge, Button, Container, Heading, StatusBadge, Text, toast } from "@medusajs/ui"
import { Link, useNavigate } from "react-router-dom"
import { SingleColumnPage } from "../../components/layout/pages"
import { useDismissOnboarding, useOnboarding, useResetOnboarding } from "../../hooks/api/onboarding"

function getStepIcon(id: string) {
  switch (id) {
    case "store":
      return <BuildingStorefront className="text-ui-fg-subtle" />
    case "products":
      return <ShoppingBag className="text-ui-fg-subtle" />
    case "shipping":
      return <TruckFast className="text-ui-fg-subtle" />
    case "payment":
      return <CreditCard className="text-ui-fg-subtle" />
    case "branding":
      return <Sparkles className="text-ui-fg-subtle" />
    default:
      return <BuildingStorefront className="text-ui-fg-subtle" />
  }
}

export const OnboardingWizard = () => {
  const navigate = useNavigate()
  const { data: onboarding, isLoading } = useOnboarding()
  const dismissMutation = useDismissOnboarding()
  const resetMutation = useResetOnboarding()

  const handleDismiss = async () => {
    try {
      await dismissMutation.mutateAsync()
      toast.success("Onboarding guide dismissed. You can reopen it anytime from Settings.")
    } catch {
      toast.error("Failed to dismiss onboarding.")
    }
  }

  const handleReset = async () => {
    try {
      await resetMutation.mutateAsync()
      toast.success("Onboarding guide restored.")
    } catch {
      toast.error("Failed to restore onboarding.")
    }
  }

  if (isLoading && !onboarding) {
    return (
      <SingleColumnPage widgets={{ after: [], before: [] }}>
        <Container className="p-8 text-center">
          <Text className="text-ui-fg-subtle">Loading store setup progress...</Text>
        </Container>
      </SingleColumnPage>
    )
  }

  const steps = onboarding?.steps || []
  const percentage = onboarding?.completion_percentage || 0
  const completedCount = onboarding?.completed_steps_count || 0
  const totalCount = onboarding?.total_steps_count || 5
  const isAllCompleted = completedCount === totalCount

  return (
    <SingleColumnPage
      widgets={{
        after: [],
        before: [],
      }}
    >
      <div className="flex flex-col gap-y-6 max-w-[800px] mx-auto w-full pb-12">
        {/* Header Banner */}
        <Container className="p-6 border border-ui-border-base rounded-2xl shadow-xs bg-ui-bg-base">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-5 border-b border-ui-border-base">
            <div className="flex flex-col gap-y-1">
              <div className="flex items-center gap-x-3">
                <Heading level="h1" className="text-xl font-bold tracking-tight text-ui-fg-base">
                  Store Setup Guide
                </Heading>
                {isAllCompleted ? (
                  <Badge size="small" color="green">
                    Ready to Sell
                  </Badge>
                ) : (
                  <Badge size="small" color="blue">
                    {completedCount} of {totalCount} completed ({percentage}%)
                  </Badge>
                )}
              </div>
              <Text size="small" className="text-ui-fg-subtle">
                Complete these essential milestones to launch your store with real payment gateways and automated courier delivery.
              </Text>
            </div>

            <div className="flex items-center gap-x-2 self-start sm:self-center">
              {onboarding?.dismissed ? (
                <Button size="small" variant="secondary" onClick={handleReset}>
                  Re-enable Guide
                </Button>
              ) : (
                <Button size="small" variant="transparent" className="text-ui-fg-muted" onClick={handleDismiss}>
                  Dismiss
                </Button>
              )}
            </div>
          </div>

          {/* Progress Bar */}
          <div className="pt-5 flex flex-col gap-y-2">
            <div className="flex items-center justify-between text-xs text-ui-fg-muted">
              <span>Overall Progression</span>
              <span className="font-semibold text-ui-fg-base">{percentage}%</span>
            </div>
            <div className="w-full bg-ui-bg-subtle rounded-full h-2.5 overflow-hidden">
              <div
                className="h-2.5 rounded-full transition-all duration-500 bg-ui-fg-interactive"
                style={{ width: `${percentage}%` }}
              />
            </div>
          </div>
        </Container>

        {/* All Completed Callout */}
        {isAllCompleted && (
          <Container className="p-6 border rounded-2xl bg-ui-bg-subtle/30 flex flex-col sm:flex-row items-center justify-between gap-4">
            <div className="flex items-center gap-x-3.5">
              <div className="h-10 w-10 rounded-full bg-ui-tag-green-bg flex items-center justify-center text-ui-tag-green-icon shrink-0">
                <CheckCircleSolid />
              </div>
              <div>
                <Heading level="h3" className="text-base font-semibold text-ui-fg-base">
                  Store Setup Complete!
                </Heading>
                <Text size="small" className="text-ui-fg-subtle">
                  Your store profile, catalog, delivery zones, and payment channels are fully configured.
                </Text>
              </div>
            </div>
            <Button size="small" variant="primary" onClick={() => navigate("/")}>
              Open Merchant Home
            </Button>
          </Container>
        )}

        {/* Steps Checklist */}
        <div className="flex flex-col gap-y-3">
          {steps.map((step, idx) => (
            <Container
              key={step.id}
              className={`p-5 border border-ui-border-base rounded-2xl shadow-xs transition-all ${
                step.completed
                  ? "bg-ui-bg-base opacity-90"
                  : "bg-ui-bg-base border-ui-border-strong ring-1 ring-ui-border-base/50"
              }`}
            >
              <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                <div className="flex items-start gap-x-4">
                  <div className="mt-0.5 flex items-center justify-center h-8 w-8 rounded-lg border bg-ui-bg-subtle/50 shrink-0">
                    {step.completed ? (
                      <CheckCircleSolid className="text-ui-tag-green-icon text-lg" />
                    ) : (
                      getStepIcon(step.id)
                    )}
                  </div>
                  <div className="flex flex-col">
                    <div className="flex items-center gap-x-2.5">
                      <Text weight="plus" className="text-sm font-semibold text-ui-fg-base">
                        {idx + 1}. {step.title}
                      </Text>
                      {step.completed ? (
                        <StatusBadge color="green">Done</StatusBadge>
                      ) : (
                        <StatusBadge color="orange">Pending</StatusBadge>
                      )}
                    </div>
                    <Text size="small" className="text-ui-fg-subtle mt-0.5">
                      {step.description}
                    </Text>
                  </div>
                </div>

                <div className="flex items-center self-end sm:self-center shrink-0">
                  <Button
                    size="small"
                    variant={step.completed ? "secondary" : "primary"}
                    asChild
                  >
                    <Link to={step.href}>
                      {step.completed ? "Review" : "Complete"}
                      <ArrowRight className="ml-1" />
                    </Link>
                  </Button>
                </div>
              </div>
            </Container>
          ))}
        </div>
      </div>
    </SingleColumnPage>
  )
}
