"use client"

import React from "react"
import * as AccordionPrimitive from "@radix-ui/react-accordion"
import { CaretDown } from "@phosphor-icons/react/dist/ssr"

export type AccordionItemProps = AccordionPrimitive.AccordionItemProps & {
  title: string
  subtitle?: string
  description?: string
  children: React.ReactNode
}

export type AccordionProps =
  | (AccordionPrimitive.AccordionSingleProps & React.RefAttributes<HTMLDivElement>)
  | (AccordionPrimitive.AccordionMultipleProps & React.RefAttributes<HTMLDivElement>)

export const Accordion: React.FC<AccordionProps> & {
  Item: React.FC<AccordionItemProps>
} = ({ children, ...props }) => {
  return (
    <AccordionPrimitive.Root {...props} className="w-full divide-y divide-[var(--hairline)]">
      {children}
    </AccordionPrimitive.Root>
  )
}

const Item: React.FC<AccordionItemProps> = ({
  title,
  subtitle,
  description,
  children,
  className = "",
  ...props
}) => {
  return (
    <AccordionPrimitive.Item
      {...props}
      className={`group py-3 ${className}`}
    >
      <AccordionPrimitive.Header className="flex">
        <AccordionPrimitive.Trigger className="flex flex-1 items-center justify-between text-left py-2 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[var(--accent-sapphire)] rounded">
          <div className="flex flex-col">
            <span className="text-xs font-semibold uppercase tracking-wider text-[var(--text-primary)]">
              {title}
            </span>
            {subtitle && (
              <span className="text-xs text-[var(--text-secondary)] mt-0.5">
                {subtitle}
              </span>
            )}
          </div>
          <CaretDown size={14} className="text-[var(--text-secondary)] transition-transform duration-200 ease-out group-data-[state=open]:rotate-180" />
        </AccordionPrimitive.Trigger>
      </AccordionPrimitive.Header>

      <AccordionPrimitive.Content className="overflow-hidden text-xs text-[var(--text-secondary)] data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down">
        <div className="pt-2 pb-4 leading-relaxed">
          {description && <p className="mb-3">{description}</p>}
          {children}
        </div>
      </AccordionPrimitive.Content>
    </AccordionPrimitive.Item>
  )
}

Accordion.Item = Item

export default Accordion
