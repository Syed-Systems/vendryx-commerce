import { useState } from "react"
import { BentoOperationsSection } from "./components/BentoOperationsSection"
import { EngineArchitectureSection } from "./components/EngineArchitectureSection"
import { Footer } from "./components/Footer"
import { HeroSection } from "./components/HeroSection"
import { Navbar } from "./components/Navbar"
import { OkfAgentsSection } from "./components/OkfAgentsSection"
import { OnboardingModal } from "./components/OnboardingModal"
import { PricingSection } from "./components/PricingSection"
import type { CurrencyCode, PlanSelection, PrepaidMonths } from "./lib/marketing-logic"

export function App() {
  const [currency, setCurrency] = useState<CurrencyCode>("USD")
  const [months, setMonths] = useState<PrepaidMonths>(1)
  const [modalOpen, setModalOpen] = useState(false)
  const [selectedPlan, setSelectedPlan] = useState<PlanSelection>("trial")

  function handleOpenOnboarding(plan: PlanSelection) {
    setSelectedPlan(plan)
    setModalOpen(true)
  }

  return (
    <div className="min-h-screen bg-[#080A0C] text-[#FFFCF8]">
      <Navbar onOpenOnboarding={handleOpenOnboarding} />

      <main>
        <HeroSection onOpenOnboarding={handleOpenOnboarding} />
        <OkfAgentsSection />
        <BentoOperationsSection />
        <EngineArchitectureSection />
        <PricingSection
          currency={currency}
          months={months}
          onCurrencyChange={setCurrency}
          onMonthsChange={setMonths}
          onSelectPlan={handleOpenOnboarding}
        />
      </main>

      <Footer onOpenOnboarding={handleOpenOnboarding} />

      <OnboardingModal
        open={modalOpen}
        plan={selectedPlan}
        currency={currency}
        months={months}
        onClose={() => setModalOpen(false)}
      />
    </div>
  )
}

export default App
