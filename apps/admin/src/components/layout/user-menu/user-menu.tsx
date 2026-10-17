import {
  BookOpen,
  CircleHalfSolid,
  EllipsisHorizontal,
  Keyboard,
  OpenRectArrowOut,
  TimelineVertical,
  User as UserIcon,
  XMark,
} from "@medusajs/icons"
import {
  Avatar,
  DropdownMenu,
  Heading,
  IconButton,
  Input,
  Kbd,
  Text,
  clx,
} from "@medusajs/ui"
import { Dialog as RadixDialog } from "radix-ui"
import { useTranslation } from "react-i18next"

import { Skeleton } from "../../common/skeleton"

import { useState } from "react"
import { Link, useLocation, useNavigate } from "react-router-dom"
import { useLogout, useMe } from "../../../hooks/api"
import { queryClient } from "../../../lib/query-client"
import { useGlobalShortcuts } from "../../../providers/keybind-provider/hooks"
import { CHANGELOG_URL, DOCS_URL } from "../../../lib/external-links"
import { useTheme } from "../../../providers/theme-provider"
import { useDocumentDirection } from "../../../hooks/use-document-direction"
import { useSafeClerk } from "../../../providers/clerk-provider"

export const UserMenu = () => {
  const { t } = useTranslation()
  const location = useLocation()
  const direction = useDocumentDirection()

  const [openMenu, setOpenMenu] = useState(false)
  const [openModal, setOpenModal] = useState(false)

  const toggleModal = () => {
    setOpenMenu(false)
    setOpenModal(!openModal)
  }

  return (
    <div>
      <DropdownMenu dir={direction} open={openMenu} onOpenChange={setOpenMenu}>
        <UserBadge />
        <DropdownMenu.Content className="w-[244px] rounded-xl p-1.5 shadow-2xl border border-ui-border-base bg-ui-bg-base/95 backdrop-blur-xl">
          <UserItem />
          <DropdownMenu.Separator className="my-1 border-ui-border-base/60" />
          <DropdownMenu.Item className="gap-x-2.5 rounded-lg py-1.5 px-2.5 text-[13px]" asChild>
            <Link to="/settings/profile" state={{ from: location.pathname }}>
              <UserIcon className="text-ui-fg-muted size-4 me-1" />
              {t("app.menus.user.profileSettings")}
            </Link>
          </DropdownMenu.Item>
          <DropdownMenu.Separator className="my-1 border-ui-border-base/60" />
          {DOCS_URL && (
            <DropdownMenu.Item className="gap-x-2.5 rounded-lg py-1.5 px-2.5 text-[13px]" asChild>
              <Link to={DOCS_URL} target="_blank">
                <BookOpen className="text-ui-fg-muted size-4 me-1" />
                {t("app.menus.user.documentation")}
              </Link>
            </DropdownMenu.Item>
          )}
          {CHANGELOG_URL && (
            <DropdownMenu.Item className="gap-x-2.5 rounded-lg py-1.5 px-2.5 text-[13px]" asChild>
              <Link to={CHANGELOG_URL} target="_blank">
                <TimelineVertical className="text-ui-fg-muted size-4 me-1" />
                {t("app.menus.user.changelog")}
              </Link>
            </DropdownMenu.Item>
          )}
          <DropdownMenu.Item className="gap-x-2.5 rounded-lg py-1.5 px-2.5 text-[13px]" onClick={toggleModal}>
            <Keyboard className="text-ui-fg-muted size-4 me-1" />
            {t("app.menus.user.shortcuts")}
          </DropdownMenu.Item>
          <ThemeToggle />
          <DropdownMenu.Separator className="my-1 border-ui-border-base/60" />
          <Logout />
        </DropdownMenu.Content>
      </DropdownMenu>
      <GlobalKeybindsModal open={openModal} onOpenChange={setOpenModal} />
    </div>
  )
}

const UserBadge = () => {
  const { user, isPending, isError, error } = useMe()
  const { user: clerkUser } = useSafeClerk()

  const name =
    [user?.first_name, user?.last_name].filter(Boolean).join(" ") ||
    clerkUser?.fullName ||
    ""
  const email = user?.email || clerkUser?.primaryEmailAddress?.emailAddress || ""
  const displayName = name || email

  const fallback = displayName ? displayName[0].toUpperCase() : null

  if (isPending) {
    return (
      <div className="p-2">
        <div className="flex select-none items-center gap-x-2.5 rounded-xl px-2 py-1.5">
          <Skeleton className="h-7 w-7 rounded-full" />
          <div className="flex flex-col gap-1">
            <Skeleton className="h-3 w-[90px]" />
            <Skeleton className="h-2.5 w-[60px]" />
          </div>
        </div>
      </div>
    )
  }

  if (isError) {
    throw error
  }

  return (
    <div className="p-2">
      <DropdownMenu.Trigger
        disabled={!user}
        className={clx(
          "bg-transparent grid w-full cursor-pointer grid-cols-[28px_1fr_auto] items-center gap-2.5 rounded-xl px-2 py-1.5 outline-none text-start transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-150",
          "hover:bg-black/[0.04] dark:hover:bg-white/[0.06]",
          "data-[state=open]:bg-black/[0.05] dark:data-[state=open]:bg-white/[0.08]",
          "focus-visible:shadow-borders-focus"
        )}
      >
        <div className="flex size-7 items-center justify-center shrink-0">
          {fallback ? (
            <Avatar size="xsmall" fallback={fallback} className="size-7 rounded-full bg-ui-bg-subtle border border-ui-border-base" />
          ) : (
            <Skeleton className="h-7 w-7 rounded-full" />
          )}
        </div>
        <div className="flex flex-col overflow-hidden text-start min-w-0">
          {displayName ? (
            <Text
              size="small"
              weight="plus"
              leading="compact"
              className="truncate text-[13px] font-semibold text-ui-fg-base"
            >
              {displayName}
            </Text>
          ) : (
            <Skeleton className="h-3 w-[80px]" />
          )}
          {email && email !== displayName && (
            <Text
              size="xsmall"
              leading="compact"
              className="truncate text-xs text-ui-fg-muted font-normal mt-0.5"
            >
              {email}
            </Text>
          )}
        </div>
        <EllipsisHorizontal className="text-ui-fg-muted size-3.5 shrink-0" />
      </DropdownMenu.Trigger>
    </div>
  )
}

const ThemeToggle = () => {
  const { t } = useTranslation()
  const { theme, setTheme } = useTheme()

  return (
    <DropdownMenu.SubMenu>
      <DropdownMenu.SubMenuTrigger
        dir="ltr"
        className="rounded-md rtl:rotate-180"
      >
        <CircleHalfSolid className="text-ui-fg-subtle me-2" />
        <span className="rtl:rotate-180">
          {t("app.menus.user.theme.label")}
        </span>
      </DropdownMenu.SubMenuTrigger>
      <DropdownMenu.SubMenuContent>
        <DropdownMenu.RadioGroup value={theme}>
          <DropdownMenu.RadioItem
            value="system"
            onClick={(e) => {
              e.preventDefault()
              setTheme("system")
            }}
          >
            {t("app.menus.user.theme.system")}
          </DropdownMenu.RadioItem>
          <DropdownMenu.RadioItem
            value="light"
            onClick={(e) => {
              e.preventDefault()
              setTheme("light")
            }}
          >
            {t("app.menus.user.theme.light")}
          </DropdownMenu.RadioItem>
          <DropdownMenu.RadioItem
            value="dark"
            onClick={(e) => {
              e.preventDefault()
              setTheme("dark")
            }}
          >
            {t("app.menus.user.theme.dark")}
          </DropdownMenu.RadioItem>
        </DropdownMenu.RadioGroup>
      </DropdownMenu.SubMenuContent>
    </DropdownMenu.SubMenu>
  )
}

const Logout = () => {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { signOut: clerkSignOut } = useSafeClerk()
  const { mutateAsync: logoutMutation } = useLogout()

  const handleLogout = async () => {
    try {
      await clerkSignOut()
    } catch {
      // ignore
    }
    try {
      await logoutMutation(undefined)
    } catch {
      // ignore
    }
    queryClient.clear()
    navigate("/login")
  }

  return (
    <DropdownMenu.Item onClick={handleLogout}>
      <div className="flex items-center gap-x-2">
        <OpenRectArrowOut className="text-ui-fg-subtle" />
        <span>{t("app.menus.actions.logout")}</span>
      </div>
    </DropdownMenu.Item>
  )
}

const GlobalKeybindsModal = (props: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) => {
  const { t } = useTranslation()
  const globalShortcuts = useGlobalShortcuts()

  const [searchValue, onSearchValueChange] = useState("")

  const searchResults = searchValue
    ? globalShortcuts.filter((shortcut) => {
        return shortcut.label.toLowerCase().includes(searchValue?.toLowerCase())
      })
    : globalShortcuts

  return (
    <RadixDialog.Root {...props}>
      <RadixDialog.Portal>
        <RadixDialog.Overlay className="bg-ui-bg-overlay fixed inset-0" />
        <RadixDialog.Content className="bg-ui-bg-subtle shadow-elevation-modal fixed left-[50%] top-[50%] flex h-full max-h-[612px] w-full max-w-[560px] translate-x-[-50%] translate-y-[-50%] flex-col divide-y overflow-hidden rounded-lg">
          <div className="flex flex-col gap-y-3 px-6 py-4">
            <div className="flex items-center justify-between">
              <div>
                <RadixDialog.Title asChild>
                  <Heading>{t("app.menus.user.shortcuts")}</Heading>
                </RadixDialog.Title>
                <RadixDialog.Description className="sr-only"></RadixDialog.Description>
              </div>
              <div className="flex items-center gap-x-2">
                <Kbd>esc</Kbd>
                <RadixDialog.Close asChild>
                  <IconButton variant="transparent" size="small">
                    <XMark />
                  </IconButton>
                </RadixDialog.Close>
              </div>
            </div>
            <div>
              <Input
                type="search"
                value={searchValue}
                onChange={(e) => onSearchValueChange(e.target.value)}
              />
            </div>
          </div>
          <div className="flex flex-col divide-y overflow-y-auto">
            {searchResults.map((shortcut, index) => {
              return (
                <div
                  key={index}
                  className="text-ui-fg-subtle flex items-center justify-between px-6 py-3"
                >
                  <Text size="small">{shortcut.label}</Text>
                  <div className="flex items-center gap-x-1">
                    {shortcut.keys.Mac?.map((key, index) => {
                      return (
                        <div className="flex items-center gap-x-1" key={index}>
                          <Kbd>{key}</Kbd>
                          {index < (shortcut.keys.Mac?.length || 0) - 1 && (
                            <span className="txt-compact-xsmall text-ui-fg-subtle">
                              {t("app.keyboardShortcuts.then")}
                            </span>
                          )}
                        </div>
                      )
                    })}
                  </div>
                </div>
              )
            })}
          </div>
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  )
}

const UserItem = () => {
  const { user, isPending, isError, error } = useMe()
  const { user: clerkUser } = useSafeClerk()

  const loaded = (!isPending && !!user) || !!clerkUser

  if (!loaded) {
    return <div></div>
  }

  const name =
    [user?.first_name, user?.last_name].filter(Boolean).join(" ") ||
    clerkUser?.fullName ||
    ""
  const email = user?.email || clerkUser?.primaryEmailAddress?.emailAddress || ""
  const fallback = name ? name[0].toUpperCase() : email ? email[0].toUpperCase() : "U"
  const avatar = user?.avatar_url || clerkUser?.imageUrl

  if (isError) {
    throw error
  }

  return (
    <div className="flex items-center gap-x-3 overflow-hidden px-2 py-1">
      <Avatar
        size="small"
        variant="rounded"
        src={avatar || undefined}
        fallback={fallback}
      />
      <div className="block w-full min-w-0 max-w-[187px] overflow-hidden whitespace-nowrap">
        <Text
          size="small"
          weight="plus"
          leading="compact"
          className="overflow-hidden text-ellipsis whitespace-nowrap"
        >
          {name || email}
        </Text>
        {!!name && (
          <Text
            size="xsmall"
            leading="compact"
            className="text-ui-fg-subtle overflow-hidden text-ellipsis whitespace-nowrap"
          >
            {email}
          </Text>
        )}
      </div>
    </div>
  )
}
