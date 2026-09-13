import { FetchError } from "@medusajs/js-sdk"
import { HttpTypes } from "@medusajs/types"
import {
  QueryKey,
  UseMutationOptions,
  UseQueryOptions,
  useMutation,
  useQuery,
} from "@tanstack/react-query"
import { sdk } from "../../lib/client"
import { queryClient } from "../../lib/query-client"
import { queryKeysFactory } from "../../lib/query-key-factory"

import { useMemo } from "react"
import { useSafeClerk } from "../../providers/clerk-provider"

const USERS_QUERY_KEY = "users" as const
const usersQueryKeys = {
  ...queryKeysFactory(USERS_QUERY_KEY),
  me: (query?: HttpTypes.AdminUserParams) =>
    [USERS_QUERY_KEY, "me", query ? { query } : undefined].filter((k) => !!k),
}

export const useMe = (
  query?: HttpTypes.AdminUserParams,
  options?: UseQueryOptions<
    HttpTypes.AdminUserResponse,
    FetchError,
    HttpTypes.AdminUserResponse,
    QueryKey
  >
) => {
  const { user: clerkUser, isSignedIn: isClerkSignedIn } = useSafeClerk()

  const { data, ...rest } = useQuery({
    queryFn: () => sdk.admin.user.me(query),
    queryKey: usersQueryKeys.me(query),
    retry: isClerkSignedIn ? false : undefined,
    ...options,
  })

  // Prioritize native backend user; fallback to Clerk authenticated user
  const effectiveUser = useMemo(() => {
    if (data?.user) {
      return data.user
    }
    if (isClerkSignedIn && clerkUser) {
      const email =
        clerkUser.primaryEmailAddress?.emailAddress ||
        clerkUser.emailAddresses?.[0]?.emailAddress ||
        "owner@clerk.local"
      return {
        id: clerkUser.id,
        email,
        first_name: clerkUser.firstName || "Storefront",
        last_name: clerkUser.lastName || "Owner",
        avatar_url: clerkUser.imageUrl || undefined,
        metadata: {
          role: "owner",
          clerk_id: clerkUser.id,
        },
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        deleted_at: null,
      } as unknown as HttpTypes.AdminUser
    }
    return undefined
  }, [data?.user, isClerkSignedIn, clerkUser])

  return {
    user: effectiveUser,
    ...rest,
  }
}

export const useUser = (
  id: string,
  query?: HttpTypes.AdminUserParams,
  options?: Omit<
    UseQueryOptions<
      HttpTypes.AdminUserResponse,
      FetchError,
      HttpTypes.AdminUserResponse,
      QueryKey
    >,
    "queryFn" | "queryKey"
  >
) => {
  const { data, ...rest } = useQuery({
    queryFn: () => sdk.admin.user.retrieve(id, query),
    queryKey: usersQueryKeys.detail(id),
    ...options,
  })

  return { ...data, ...rest }
}

export const useUsers = (
  query?: HttpTypes.AdminUserListParams,
  options?: Omit<
    UseQueryOptions<
      HttpTypes.AdminUserListResponse,
      FetchError,
      HttpTypes.AdminUserListResponse,
      QueryKey
    >,
    "queryFn" | "queryKey"
  >
) => {
  const { data, ...rest } = useQuery({
    queryFn: () => sdk.admin.user.list(query),
    queryKey: usersQueryKeys.list(query),
    ...options,
  })

  return { ...data, ...rest }
}

export const useUpdateUser = (
  id: string,
  query?: HttpTypes.AdminUserParams,
  options?: UseMutationOptions<
    HttpTypes.AdminUserResponse,
    FetchError,
    HttpTypes.AdminUpdateUser,
    QueryKey
  >
) => {
  return useMutation({
    mutationFn: (payload) => sdk.admin.user.update(id, payload, query),
    onSuccess: (data, variables, context) => {
      queryClient.invalidateQueries({ queryKey: usersQueryKeys.detail(id) })
      queryClient.invalidateQueries({ queryKey: usersQueryKeys.lists() })

      // We invalidate the me query in case the user updates their own profile
      queryClient.invalidateQueries({ queryKey: usersQueryKeys.me() })

      options?.onSuccess?.(data, variables, context)
    },
    ...options,
  })
}

export const useDeleteUser = (
  id: string,
  options?: UseMutationOptions<
    HttpTypes.AdminUserDeleteResponse,
    FetchError,
    void
  >
) => {
  return useMutation({
    mutationFn: () => sdk.admin.user.delete(id),
    onSuccess: (data, variables, context) => {
      queryClient.invalidateQueries({ queryKey: usersQueryKeys.detail(id) })
      queryClient.invalidateQueries({ queryKey: usersQueryKeys.lists() })

      // We invalidate the me query in case the user updates their own profile
      queryClient.invalidateQueries({ queryKey: usersQueryKeys.me() })

      options?.onSuccess?.(data, variables, context)
    },
    ...options,
  })
}
