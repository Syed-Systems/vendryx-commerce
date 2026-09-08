import {
  DndContext,
  DragEndEvent,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
} from "@dnd-kit/core"
import {
  SortableContext,
  arrayMove,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable"
import { CSS } from "@dnd-kit/utilities"
import { DotsSix, Eye, EyeSlash, TriangleRightMini } from "@medusajs/icons"
import { IconButton, Kbd, Text, clx } from "@medusajs/ui"
import { Collapsible as RadixCollapsible } from "radix-ui"
import {
  PropsWithChildren,
  ReactNode,
  useCallback,
  useEffect,
  useState,
} from "react"
import { useTranslation } from "react-i18next"
import { NavLink, To, useLocation } from "react-router-dom"
import { useLayoutEdit } from "../../../hooks/use-layout-edit"
import { useGlobalShortcuts } from "../../../providers/keybind-provider/hooks"
import { ConditionalTooltip } from "../../common/conditional-tooltip"

type ItemType = "core" | "extension" | "setting"

type NestedItemProps = {
  label: string
  to: string
  translationNs?: string
}

export type INavItem = {
  icon?: ReactNode
  label: string
  to: string
  items?: NestedItemProps[]
  type?: ItemType
  from?: string
  nested?: string
  translationNs?: string
}

const BASE_NAV_LINK_CLASSES =
  "group text-ui-fg-subtle transition-all duration-150 hover:text-ui-fg-base hover:bg-black/[0.04] dark:hover:bg-white/[0.06] flex items-center gap-x-2.5 rounded-lg h-[34px] px-2.5 outline-none [&_svg]:text-ui-fg-muted hover:[&_svg]:text-ui-fg-subtle focus-visible:shadow-borders-focus select-none"
const ACTIVE_NAV_LINK_CLASSES =
  "bg-black/[0.07] text-ui-fg-base font-semibold shadow-xs dark:bg-white/[0.12] dark:text-white hover:bg-black/[0.08] dark:hover:bg-white/[0.16] [&_svg]:text-ui-fg-base dark:[&_svg]:text-white"
const NESTED_NAV_LINK_CLASSES =
  "h-7 px-2.5 py-1 text-[12px] font-normal text-ui-fg-muted hover:text-ui-fg-base rounded-md w-full"
const SETTING_NAV_LINK_CLASSES = "h-[32px] px-2.5 py-1.5"

// Prefix match that respects path-segment boundaries, so that a route like
// "/inventory-sources" is not considered a match for "/inventory". This keeps
// nested-route highlighting (e.g. "/inventory/123") while avoiding false
// positives for sibling routes that merely share a string prefix.
const isPathActive = (pathname: string, to: string) => {
  return pathname === to || pathname.startsWith(to + "/")
}

const getIsOpen = (
  to: string,
  items: NestedItemProps[] | undefined,
  pathname: string
) => {
  return [to, ...(items?.map((i) => i.to) ?? [])].some((p) =>
    isPathActive(pathname, p)
  )
}

const NavItemTooltip = ({
  to,
  children,
}: PropsWithChildren<{ to: string }>) => {
  const { t } = useTranslation()
  const globalShortcuts = useGlobalShortcuts()
  const shortcut = globalShortcuts.find((s) => s.to === to)

  return (
    <ConditionalTooltip
      showTooltip={!!shortcut}
      maxWidth={9999} // Don't limit the width of the tooltip
      content={
        <div className="txt-compact-xsmall flex h-5 items-center justify-between gap-x-2 whitespace-nowrap">
          <span>{shortcut?.label}</span>
          <div className="flex items-center gap-x-1">
            {shortcut?.keys.Mac?.map((key, index) => (
              <div className="flex items-center gap-x-1" key={index}>
                <Kbd key={key}>{key}</Kbd>
                {index < (shortcut.keys.Mac?.length || 0) - 1 && (
                  <span className="text-ui-fg-muted txt-compact-xsmall">
                    {t("app.keyboardShortcuts.then")}
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>
      }
      side="right"
      delayDuration={1500}
    >
      <div className="w-full">{children}</div>
    </ConditionalTooltip>
  )
}

type SubItemLinkProps = {
  item: NestedItemProps
  isSetting: boolean
  navLinkClassNames: (props: {
    to: string
    isActive: boolean
    isNested?: boolean
    isSetting?: boolean
  }) => string
  getLinkTarget: (target: string) => To
}

const SubItemLink = ({
  item,
  isSetting,
  navLinkClassNames,
  getLinkTarget,
}: SubItemLinkProps) => {
  const { t } = useTranslation(item.translationNs as any)
  const itemLabel: string = item.translationNs ? t(item.label) : item.label

  return (
    <NavItemTooltip to={item.to}>
      <NavLink
        to={getLinkTarget(item.to)}
        end
        className={({ isActive }) => {
          return clx(
            navLinkClassNames({
              to: item.to,
              isActive,
              isSetting,
              isNested: true,
            })
          )
        }}
      >
        <Text size="small" weight="plus" leading="compact">
          {itemLabel}
        </Text>
      </NavLink>
    </NavItemTooltip>
  )
}

const NavItemSubItem = (props: SubItemLinkProps) => {
  return (
    <li className="flex h-7 items-center">
      <SubItemLink {...props} />
    </li>
  )
}

const SortableSubItem = ({
  id,
  hidden,
  onToggleHidden,
  ...props
}: SubItemLinkProps & {
  id: string
  hidden: boolean
  onToggleHidden: () => void
}) => {
  const { t } = useTranslation()
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id })

  const style = {
    transform: CSS.Translate.toString(transform),
    transition,
  }

  return (
    <li
      ref={setNodeRef}
      style={style}
      className={clx(
        "flex h-7 items-center gap-x-1",
        isDragging && "opacity-50"
      )}
    >
      <button
        type="button"
        className="text-ui-fg-muted cursor-grab touch-none rounded p-0.5 focus:outline-none [&>svg]:h-3.5 [&>svg]:w-3.5"
        {...attributes}
        {...listeners}
        aria-label={t("layout.dragToReorder")}
      >
        <DotsSix />
      </button>
      <IconButton
        size="2xsmall"
        variant="transparent"
        onClick={onToggleHidden}
        aria-label={hidden ? t("actions.show") : t("actions.hide")}
        className="h-3.5 w-3.5 p-0.5"
      >
        {hidden ? <EyeSlash /> : <Eye />}
      </IconButton>
      <div className={clx("min-w-0 flex-1", hidden && "opacity-30 grayscale")}>
        <SubItemLink {...props} />
      </div>
    </li>
  )
}

/**
 * Renders a route's nested children, applying the saved order. In layout edit
 * mode they become a self-contained vertical sortable (its own `DndContext`, so
 * it doesn't interfere with the composer's entry-level drag), scoped to this
 * route — child ids embed the parent path, so moves never cross routes.
 */
const NavSubItems = ({
  parentTo,
  items,
  isSetting,
  navLinkClassNames,
  getLinkTarget,
}: {
  parentTo: string
  items: NestedItemProps[]
  isSetting: boolean
  navLinkClassNames: SubItemLinkProps["navLinkClassNames"]
  getLinkTarget: SubItemLinkProps["getLinkTarget"]
}) => {
  const { editMode, orderChildren, setChildrenOrder, isHidden, toggleHidden } =
    useLayoutEdit()
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } })
  )
  const childId = useCallback(
    (to: string) => `nav-child:${parentTo}:${to}`,
    [parentTo]
  )

  const ordered = orderChildren(items, (it) => childId(it.to))
  const shared = { isSetting, navLinkClassNames, getLinkTarget }

  if (!editMode) {
    return (
      <>
        {ordered
          .filter((item) => !isHidden(childId(item.to)))
          .map((item) => (
            <NavItemSubItem key={item.to} item={item} {...shared} />
          ))}
      </>
    )
  }

  const ids = ordered.map((it) => childId(it.to))

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    if (!over || active.id === over.id) {
      return
    }
    const oldIndex = ids.indexOf(active.id as string)
    const newIndex = ids.indexOf(over.id as string)
    if (oldIndex < 0 || newIndex < 0) {
      return
    }
    setChildrenOrder(arrayMove(ids, oldIndex, newIndex))
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragEnd={handleDragEnd}
    >
      <SortableContext items={ids} strategy={verticalListSortingStrategy}>
        {ordered.map((item) => (
          <SortableSubItem
            key={item.to}
            id={childId(item.to)}
            item={item}
            hidden={isHidden(childId(item.to))}
            onToggleHidden={() => toggleHidden(childId(item.to))}
            {...shared}
          />
        ))}
      </SortableContext>
    </DndContext>
  )
}

export const NavItem = ({
  icon,
  label,
  to,
  items,
  type = "core",
  from,
  translationNs,
}: INavItem) => {
  const { t } = useTranslation(translationNs as any)
  const { pathname, search } = useLocation()
  const [open, setOpen] = useState(getIsOpen(to, items, pathname))
  // While the layout is being edited, force the nested list open so every
  // route's children are visible and reorderable, not just the active route's.
  const { editMode } = useLayoutEdit()

  // Use translation if translationNs is provided, otherwise use label as-is
  const displayLabel: string = translationNs ? t(label) : label

  useEffect(() => {
    setOpen(getIsOpen(to, items, pathname))
  }, [pathname, to, items])

  const navLinkClassNames = useCallback(
    ({
      to,
      isActive,
      isNested = false,
      isSetting = false,
    }: {
      to: string
      isActive: boolean
      isNested?: boolean
      isSetting?: boolean
    }) => {
      if (["core", "setting"].includes(type)) {
        isActive = isPathActive(pathname, to)
      }

      return clx(BASE_NAV_LINK_CLASSES, {
        [NESTED_NAV_LINK_CLASSES]: isNested,
        [ACTIVE_NAV_LINK_CLASSES]: isActive && !editMode,
        [SETTING_NAV_LINK_CLASSES]: isSetting,
      })
    },
    [type, pathname, editMode]
  )

  const getLinkTarget = useCallback(
    (target: string) => {
      if (pathname === target && search) {
        return { pathname: target, search }
      }

      return target
    },
    [pathname, search]
  )

  const isSetting = type === "setting"
  const hasSubItems = items && items.length > 0

  return (
    <div>
      <div className="relative flex items-center group">
        <NavItemTooltip to={to}>
          <NavLink
            to={getLinkTarget(to)}
            end={items?.some((i) => i.to === pathname)}
            state={
              from
                ? {
                    from,
                  }
                : undefined
            }
            className={({ isActive }) => {
              return clx(
                navLinkClassNames({ isActive, isSetting, to }),
                "flex-1",
                hasSubItems && "pe-7"
              )
            }}
          >
            {icon && (
              <div className="flex size-5 items-center justify-center shrink-0 [&>svg]:size-[18px]">
                <Icon icon={icon} type={type} />
              </div>
            )}
            <Text size="small" weight="plus" leading="compact" className="truncate text-[13px]">
              {displayLabel}
            </Text>
          </NavLink>
        </NavItemTooltip>

        {hasSubItems && (
          <button
            type="button"
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              setOpen((prev) => !prev)
            }}
            aria-label={open ? "Collapse" : "Expand"}
            className="absolute end-1.5 flex size-5 items-center justify-center rounded-md text-ui-fg-muted hover:text-ui-fg-base hover:bg-black/[0.06] dark:hover:bg-white/[0.08] transition-colors"
          >
            <TriangleRightMini
              className={clx(
                "size-3.5 transition-transform duration-200 text-ui-fg-muted",
                open && "rotate-90 text-ui-fg-subtle"
              )}
            />
          </button>
        )}
      </div>

      {hasSubItems && (
        <RadixCollapsible.Root open={editMode || open} onOpenChange={setOpen}>
          <RadixCollapsible.Content>
            <div className="ml-5 pl-2.5 border-l border-ui-border-base/50 flex flex-col gap-y-0.5 my-1">
              <ul className="flex flex-col gap-y-0.5">
                <NavSubItems
                  parentTo={to}
                  items={items}
                  isSetting={isSetting}
                  navLinkClassNames={navLinkClassNames}
                  getLinkTarget={getLinkTarget}
                />
              </ul>
            </div>
          </RadixCollapsible.Content>
        </RadixCollapsible.Root>
      )}
    </div>
  )
}

const Icon = ({ icon, type }: { icon?: ReactNode; type: ItemType }) => {
  if (!icon) {
    return null
  }

  return type === "extension" ? (
    <div className="shadow-borders-base bg-ui-bg-base flex h-5 w-5 items-center justify-center rounded-[4px]">
      <div className="h-[15px] w-[15px] overflow-hidden rounded-sm">{icon}</div>
    </div>
  ) : (
    icon
  )
}
