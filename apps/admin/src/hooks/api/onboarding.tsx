import { useMutation, useQuery } from "@tanstack/react-query"
import { sdk } from "../../lib/client"
import { queryClient } from "../../lib/query-client"

export interface OnboardingStep {
  id: string
  title: string
  description: string
  completed: boolean
  href: string
}

export interface OnboardingStatus {
  is_completed: boolean
  dismissed: boolean
  completion_percentage: number
  completed_steps_count: number
  total_steps_count: number
  steps: OnboardingStep[]
}

export const ONBOARDING_QUERY_KEY = "onboarding" as const

export const useOnboarding = () => {
  return useQuery({
    queryKey: [ONBOARDING_QUERY_KEY],
    queryFn: async () => {
      const res = await sdk.client.fetch<{ onboarding: OnboardingStatus }>(
        "/admin/onboarding",
        { method: "GET" }
      )
      return res.onboarding
    },
  })
}

export const useDismissOnboarding = () => {
  return useMutation({
    mutationFn: async () => {
      const res = await sdk.client.fetch<{ dismissed: boolean }>(
        "/admin/onboarding/dismiss",
        { method: "POST" }
      )
      return res
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [ONBOARDING_QUERY_KEY] })
    },
  })
}

export const useResetOnboarding = () => {
  return useMutation({
    mutationFn: async () => {
      const res = await sdk.client.fetch<{ dismissed: boolean }>(
        "/admin/onboarding/reset",
        { method: "POST" }
      )
      return res
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [ONBOARDING_QUERY_KEY] })
    },
  })
}
