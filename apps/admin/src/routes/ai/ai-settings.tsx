import { Button, Container, Heading, Input, Text, toast } from "@medusajs/ui"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { FormEvent, useEffect, useRef, useState } from "react"
import { SingleColumnPage } from "../../components/layout/pages"
import { sdk } from "../../lib/client"

type AiConfiguration = {
  id: string
  kind: string
  account_id: string
  configured: boolean
  enabled: boolean
  version: number
  webhook_id: string | null
}

type KnowledgeEntry = {
  id: string
  title: string
  category: string | null
  content: string
}

type ChatOffer = {
  id: string
  variant_id: string
  currency_code: string
  chat_unit_price: string
  max_quantity: number
  starts_at: string | null
  ends_at: string | null
  enabled: boolean
  version: number
}

type StoreIntelligence = {
  generated_at: string
  products_truncated: boolean
  categories_truncated: boolean
  recommendations_truncated: boolean
  recommendations: Array<{
    id: string
    review_decision: string | null
    subject_type: string
    subject_id: string
    subject_label: string
    action: string
    rationale: string
    current_orders: number
    confidence: string
    evidence: string
    knowledge_lookup: string
    knowledge_sources: Array<{
      entry_id: string
      title: string
      updated_at: string
    }>
    owner_approval_required: boolean
  }>
  products: Array<{
    product_id: string
    product_name: string
    category: string | null
    current_units: number
    comparison_units: number
    current_orders: number
    change_basis_points: number | null
    direction: string
    confidence: string
  }>
  categories: Array<{
    category: string
    archetype_rules_version: string
    recommended_archetype: string
    recommended_image_aspect: string
    archetype_basis: string
    current_units: number
    comparison_units: number
    change_basis_points: number | null
    direction: string
    confidence: string
  }>
}

type StoreIntelligenceRanking = {
  status: string
  ranked: Array<{ recommendation_id: string; uncertainty: string }>
  evidence_citations: Array<{ recommendation_id: string; source_id: string }>
  abstention_reason: string | null
}

type StoreIntelligenceReviewAudit = {
  request_id: string
  decision: string
  actor_id: string
  evidence_source: string
  rule_version: string
  created_at: string
}

type SearchConsoleReport = {
  source: string
  property: string
  window_started_on: string
  window_ended_on: string
  window_timezone: string
  ingested_at: string
  coverage: "complete" | "partial" | "unavailable"
  rows_received: number
  observations: Array<{
    metric: string
    value: string | number
    unit: string
    denominator: string | number | null
    sample_size: number | null
  }>
}

type MetaInsightsReport = {
  source: string
  account_id: string
  window_started_on: string
  window_ended_on: string
  window_timezone: string
  ingested_at: string
  coverage: "complete" | "partial" | "unavailable"
  rows_received: number
  observations: SearchConsoleReport["observations"]
}

type WhatsAppInsightsReport = {
  source: string
  account_id: string
  window_started_on: string
  window_ended_on: string
  window_timezone: string
  ingested_at: string
  coverage: "partial" | "unavailable"
  rows_received: number
  inbound_events: number
  consented_conversations: number
}

type SearchConsoleSnapshot = {
  source: string
  account_id: string
  scope: string
  metric: string
  definition: string
  unit: string
  value: string | number
  denominator: string | number | null
  sample_size: number | null
  window_started_on: string
  window_ended_on: string
  window_timezone: string
  ingested_at: string
  coverage: "complete" | "partial" | "unavailable"
}

type SearchConsoleChange = {
  metric: string
  previous_value: string | number
  current_value: string | number
  unit: string
  change_basis_points: string | number
  previous_window_started_on: string
  previous_window_ended_on: string
  current_window_started_on: string
  current_window_ended_on: string
}

type SourceConnectionAudit = {
  action: "connected" | "updated" | "disabled" | "disconnected"
  actor_id: string
  created_at: string
}

const CONFIG_QUERY = ["admin", "ai", "configurations"]
const KNOWLEDGE_QUERY = ["admin", "ai", "knowledge"]
const CHAT_OFFERS_QUERY = ["admin", "ai", "chat-offers"]
const STORE_INTELLIGENCE_QUERY = ["admin", "ai", "store-intelligence"]

const toLocalDateTimeInput = (value: string | null) => {
  if (!value) return ""
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ""
  date.setMinutes(date.getMinutes() - date.getTimezoneOffset())
  return date.toISOString().slice(0, 16)
}

export const AiSettings = () => {
  const queryClient = useQueryClient()
  const [kind, setKind] = useState("llm")
  const [accountId, setAccountId] = useState("default")
  const [provider, setProvider] = useState("deepseek")
  const [model, setModel] = useState("deepseek-chat")
  const [secrets, setSecrets] = useState({ api_key: "", app_secret: "", verify_token: "", access_token: "", client_id: "", client_secret: "", refresh_token: "" })
  const [enabled, setEnabled] = useState(true)
  const [title, setTitle] = useState("")
  const [category, setCategory] = useState("")
  const [content, setContent] = useState("")
  const [offerVariant, setOfferVariant] = useState("")
  const [offerCurrency, setOfferCurrency] = useState("BDT")
  const [offerPrice, setOfferPrice] = useState("")
  const [offerMaxQuantity, setOfferMaxQuantity] = useState("1")
  const [offerStartsAt, setOfferStartsAt] = useState("")
  const [offerEndsAt, setOfferEndsAt] = useState("")
  const [offerEnabled, setOfferEnabled] = useState(false)
  const [editingOfferId, setEditingOfferId] = useState<string | null>(null)
  const [modelRanking, setModelRanking] = useState<StoreIntelligenceRanking | null>(null)
  const [reviewHistoryId, setReviewHistoryId] = useState<string | null>(null)
  const [searchConsoleProperty, setSearchConsoleProperty] = useState("")
  const [searchConsoleEndDate, setSearchConsoleEndDate] = useState(() => new Date().toISOString().slice(0, 10))
  const [searchConsoleStartDate, setSearchConsoleStartDate] = useState(() => new Date(Date.now() - 13 * 24 * 60 * 60 * 1000).toISOString().slice(0, 10))
  const [searchConsoleReport, setSearchConsoleReport] = useState<SearchConsoleReport | null>(null)
  const [metaInsightsChannel, setMetaInsightsChannel] = useState("messenger")
  const [metaInsightsAccount, setMetaInsightsAccount] = useState("")
  const [metaInsightsEndDate, setMetaInsightsEndDate] = useState(() => new Date().toISOString().slice(0, 10))
  const [metaInsightsStartDate, setMetaInsightsStartDate] = useState(() => new Date(Date.now() - 13 * 24 * 60 * 60 * 1000).toISOString().slice(0, 10))
  const [metaInsightsReport, setMetaInsightsReport] = useState<MetaInsightsReport | null>(null)
  const [whatsAppInsightsAccount, setWhatsAppInsightsAccount] = useState("")
  const [whatsAppInsightsEndDate, setWhatsAppInsightsEndDate] = useState(() => new Date().toISOString().slice(0, 10))
  const [whatsAppInsightsStartDate, setWhatsAppInsightsStartDate] = useState(() => new Date(Date.now() - 13 * 24 * 60 * 60 * 1000).toISOString().slice(0, 10))
  const [whatsAppInsightsReport, setWhatsAppInsightsReport] = useState<WhatsAppInsightsReport | null>(null)
  const handledSearchConsoleOAuth = useRef<string | null>(null)
  const searchConsoleSnapshots = useQuery({
    queryKey: ["search-console-snapshots", searchConsoleProperty],
    enabled: Boolean(searchConsoleProperty),
    queryFn: () => sdk.client.fetch<{ snapshots: SearchConsoleSnapshot[]; changes: SearchConsoleChange[] }>(
      `/admin/ai/search-console/snapshots?property=${encodeURIComponent(searchConsoleProperty)}`
    ),
  })
  const searchConsoleAudit = useQuery({
    queryKey: ["search-console-audit", searchConsoleProperty],
    enabled: Boolean(searchConsoleProperty),
    queryFn: () => sdk.client.fetch<{ history: SourceConnectionAudit[] }>(
      `/admin/ai/search-console/audit?property=${encodeURIComponent(searchConsoleProperty)}`
    ),
  })
  const metaInsightsSnapshots = useQuery({
    queryKey: ["meta-insights-snapshots", metaInsightsChannel, metaInsightsAccount],
    enabled: Boolean(metaInsightsAccount),
    queryFn: () => sdk.client.fetch<{ snapshots: SearchConsoleSnapshot[]; changes: SearchConsoleChange[] }>(
      `/admin/ai/meta-insights/snapshots?channel=${encodeURIComponent(metaInsightsChannel)}&account_id=${encodeURIComponent(metaInsightsAccount)}`
    ),
  })
  const metaInsightsAudit = useQuery({
    queryKey: ["meta-insights-audit", metaInsightsChannel, metaInsightsAccount],
    enabled: Boolean(metaInsightsAccount),
    queryFn: () => sdk.client.fetch<{ history: SourceConnectionAudit[] }>(
      `/admin/ai/meta-insights/audit?channel=${encodeURIComponent(metaInsightsChannel)}&account_id=${encodeURIComponent(metaInsightsAccount)}`
    ),
  })

  const configurations = useQuery({
    queryKey: CONFIG_QUERY,
    queryFn: () => sdk.client.fetch<{ configurations: AiConfiguration[] }>("/admin/ai/configurations"),
  })
  const knowledge = useQuery({
    queryKey: KNOWLEDGE_QUERY,
    queryFn: () => sdk.client.fetch<{ knowledge: KnowledgeEntry[] }>("/admin/ai/knowledge?limit=50"),
  })
  const chatOffers = useQuery({
    queryKey: CHAT_OFFERS_QUERY,
    queryFn: () => sdk.client.fetch<{ chat_offers: ChatOffer[] }>("/admin/ai/chat-offers"),
  })
  const storeIntelligence = useQuery({
    queryKey: STORE_INTELLIGENCE_QUERY,
    queryFn: () => sdk.client.fetch<{ store_intelligence: StoreIntelligence }>("/admin/ai/store-intelligence"),
  })
  const reviewHistory = useQuery({
    queryKey: [...STORE_INTELLIGENCE_QUERY, "review-history", reviewHistoryId],
    enabled: reviewHistoryId !== null,
    queryFn: () => {
      if (!reviewHistoryId) throw new Error("Select a recommendation to view its review history.")
      return sdk.client.fetch<{ history: StoreIntelligenceReviewAudit[] }>(
        `/admin/ai/store-intelligence/recommendations/${encodeURIComponent(reviewHistoryId)}/review-history?limit=20`
      )
    },
  })
  const rankStoreOptions = useMutation({
    mutationFn: () => sdk.client.fetch<{ model_ranking: StoreIntelligenceRanking }>("/admin/ai/store-intelligence?rank_local=true"),
    onSuccess: (result) => setModelRanking(result.model_ranking),
    onError: (error: Error) => toast.error("Local ranking was unavailable", { description: error.message }),
  })
  const reviewRecommendation = useMutation({
    mutationFn: ({ id, decision, requestId }: { id: string; decision: string; requestId: string }) =>
      sdk.client.fetch(`/admin/ai/store-intelligence/recommendations/${encodeURIComponent(id)}/review`, {
        method: "POST",
        body: { decision, request_id: requestId },
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: STORE_INTELLIGENCE_QUERY })
      await queryClient.invalidateQueries({ queryKey: [...STORE_INTELLIGENCE_QUERY, "review-history"] })
      toast.success("Recommendation review saved", { description: "This records your review only; it does not change products, prices, or storefront settings." })
    },
    onError: (error: Error) => toast.error("Could not save recommendation review", { description: error.message }),
  })

  const saveChatOffer = useMutation({
    mutationFn: () => sdk.client.fetch("/admin/ai/chat-offers", {
      method: "POST",
      body: {
        ...(editingOfferId ? { id: editingOfferId } : {}),
        variant_id: offerVariant.trim(),
        currency_code: offerCurrency.trim().toUpperCase(),
        chat_unit_price: offerPrice.trim(),
        max_quantity: Number(offerMaxQuantity),
        starts_at: offerStartsAt ? new Date(offerStartsAt).toISOString() : null,
        ends_at: offerEndsAt ? new Date(offerEndsAt).toISOString() : null,
        enabled: offerEnabled,
      },
    }),
    onSuccess: async () => {
      setOfferVariant("")
      setOfferPrice("")
      setOfferMaxQuantity("1")
      setOfferStartsAt("")
      setOfferEndsAt("")
      setOfferEnabled(false)
      setEditingOfferId(null)
      await queryClient.invalidateQueries({ queryKey: CHAT_OFFERS_QUERY })
      toast.success("Chat price saved", { description: "The offer applies only to this variant and currency." })
    },
    onError: (error: Error) => toast.error("Could not save chat price", { description: error.message }),
  })

  const deleteChatOffer = useMutation({
    mutationFn: (id: string) => sdk.client.fetch(`/admin/ai/chat-offers/${id}`, { method: "DELETE" }),
    onSuccess: (_, id) => {
      if (editingOfferId === id) cancelChatOfferEdit()
      return queryClient.invalidateQueries({ queryKey: CHAT_OFFERS_QUERY })
    },
    onError: (error: Error) => toast.error("Could not disable chat price", { description: error.message }),
  })

  const editChatOffer = (offer: ChatOffer) => {
    setEditingOfferId(offer.id)
    setOfferVariant(offer.variant_id)
    setOfferCurrency(offer.currency_code)
    setOfferPrice(offer.chat_unit_price)
    setOfferMaxQuantity(String(offer.max_quantity))
    setOfferStartsAt(toLocalDateTimeInput(offer.starts_at))
    setOfferEndsAt(toLocalDateTimeInput(offer.ends_at))
    setOfferEnabled(offer.enabled)
  }

  const cancelChatOfferEdit = () => {
    setEditingOfferId(null)
    setOfferVariant("")
    setOfferPrice("")
    setOfferMaxQuantity("1")
    setOfferStartsAt("")
    setOfferEndsAt("")
    setOfferEnabled(false)
  }

  const saveConfiguration = useMutation({
    mutationFn: () => sdk.client.fetch("/admin/ai/configurations", {
      method: "POST",
      body: {
        kind,
        account_id: kind === "llm" ? "default" : accountId,
        enabled,
        secrets: kind === "llm"
          ? { provider, model, api_key: provider === "ollama" ? "local-ollama" : secrets.api_key }
          : kind === "search_console"
            ? { client_id: secrets.client_id, client_secret: secrets.client_secret, refresh_token: secrets.refresh_token }
            : kind.startsWith("meta_insights_")
              ? { app_secret: secrets.app_secret, access_token: secrets.access_token }
              : { app_secret: secrets.app_secret, verify_token: secrets.verify_token, access_token: secrets.access_token },
      },
    }),
    onSuccess: async () => {
      setSecrets({ api_key: "", app_secret: "", verify_token: "", access_token: "", client_id: "", client_secret: "", refresh_token: "" })
      await queryClient.invalidateQueries({ queryKey: CONFIG_QUERY })
      toast.success("AI configuration saved", { description: "Secret values are encrypted and will not be shown again." })
    },
    onError: (error: Error) => toast.error("Could not save configuration", { description: error.message }),
  })

  const disconnectSearchConsole = useMutation({
    mutationFn: (property: string) => sdk.client.fetch("/admin/ai/search-console/configuration", {
      method: "DELETE",
      body: { property },
    }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: CONFIG_QUERY })
      await queryClient.invalidateQueries({ queryKey: ["search-console-snapshots"] })
      await queryClient.invalidateQueries({ queryKey: ["search-console-audit"] })
      setSearchConsoleReport(null)
      setSearchConsoleProperty("")
      toast.success("Search Console disconnected", { description: "The encrypted refresh token was removed." })
    },
    onError: (error: Error) => toast.error("Could not disconnect Search Console", { description: error.message }),
  })

  const disconnectMetaInsights = useMutation({
    mutationFn: ({ channel, account_id }: { channel: string; account_id: string }) =>
      sdk.client.fetch("/admin/ai/meta-insights/configuration", {
        method: "DELETE",
        body: { channel, account_id },
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: CONFIG_QUERY })
      await queryClient.invalidateQueries({ queryKey: ["meta-insights-snapshots"] })
      await queryClient.invalidateQueries({ queryKey: ["meta-insights-audit"] })
      setMetaInsightsAccount("")
      setMetaInsightsReport(null)
      toast.success("Meta insights disconnected", { description: "The separate insights token and saved observations were removed." })
    },
    onError: (error: Error) => toast.error("Could not disconnect Meta insights", { description: error.message }),
  })

  const setMetaInsightsEnabled = useMutation({
    mutationFn: ({ item, enabled: nextEnabled }: { item: AiConfiguration; enabled: boolean }) =>
      sdk.client.fetch("/admin/ai/configurations", {
        method: "POST",
        body: { kind: item.kind, account_id: item.account_id, enabled: nextEnabled, secrets: {} },
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: CONFIG_QUERY })
      await queryClient.invalidateQueries({ queryKey: ["meta-insights-snapshots"] })
      toast.success("Meta insights connection updated")
    },
    onError: (error: Error) => toast.error("Could not update Meta insights", { description: error.message }),
  })

  const startSearchConsoleOAuth = useMutation({
    mutationFn: (property: string) => sdk.client.fetch<{ authorization_url: string }>("/admin/ai/search-console/oauth/start", {
      method: "POST",
      body: { property },
    }),
    onSuccess: ({ authorization_url }) => window.location.assign(authorization_url),
    onError: (error: Error) => toast.error("Could not start Search Console authorization", { description: error.message }),
  })

  const completeSearchConsoleOAuth = useMutation({
    mutationFn: ({ code, state }: { code: string; state: string }) =>
      sdk.client.fetch<{ configuration: AiConfiguration }>("/admin/ai/search-console/oauth/complete", {
        method: "POST",
        body: { code, state },
      }),
    onSuccess: async () => {
      window.history.replaceState({}, "", window.location.pathname)
      await queryClient.invalidateQueries({ queryKey: CONFIG_QUERY })
      await queryClient.invalidateQueries({ queryKey: ["search-console-audit"] })
      toast.success("Google Search Console connected", { description: "Read-only property access is encrypted and ready to fetch." })
    },
    onError: (error: Error) => {
      window.history.replaceState({}, "", window.location.pathname)
      toast.error("Search Console authorization failed", { description: error.message })
    },
  })

  useEffect(() => {
    const params = new URLSearchParams(window.location.search)
    const code = params.get("code")
    const state = params.get("state")
    const providerError = params.get("error")
    if (providerError) {
      window.history.replaceState({}, "", window.location.pathname)
      toast.error("Google Search Console authorization was not granted")
      return
    }
    if (!code || !state || handledSearchConsoleOAuth.current === state) return
    handledSearchConsoleOAuth.current = state
    completeSearchConsoleOAuth.mutate({ code, state })
  }, [])

  const fetchSearchConsole = useMutation({
    mutationFn: () => {
      const params = new URLSearchParams({
        property: searchConsoleProperty,
        start_date: searchConsoleStartDate,
        end_date: searchConsoleEndDate,
      })
      return sdk.client.fetch<{ search_console: SearchConsoleReport }>(`/admin/ai/search-console/insights?${params.toString()}`)
    },
    onSuccess: async ({ search_console }) => {
      setSearchConsoleReport(search_console)
      await queryClient.invalidateQueries({ queryKey: ["search-console-snapshots", searchConsoleProperty] })
    },
    onError: (error: Error) => toast.error("Could not load Search Console data", { description: error.message }),
  })

  const fetchMetaInsights = useMutation({
    mutationFn: () => {
      const params = new URLSearchParams({
        channel: metaInsightsChannel,
        account_id: metaInsightsAccount,
        start_date: metaInsightsStartDate,
        end_date: metaInsightsEndDate,
      })
      return sdk.client.fetch<{ meta_insights: MetaInsightsReport }>(`/admin/ai/meta-insights?${params.toString()}`)
    },
    onSuccess: async ({ meta_insights }) => {
      setMetaInsightsReport(meta_insights)
      await queryClient.invalidateQueries({ queryKey: ["meta-insights-snapshots", metaInsightsChannel, metaInsightsAccount] })
    },
    onError: (error: Error) => toast.error("Could not load Meta insights", { description: error.message }),
  })

  const fetchWhatsAppInsights = useMutation({
    mutationFn: () => {
      const params = new URLSearchParams({
        account_id: whatsAppInsightsAccount,
        start_date: whatsAppInsightsStartDate,
        end_date: whatsAppInsightsEndDate,
      })
      return sdk.client.fetch<{ whatsapp_insights: WhatsAppInsightsReport }>(`/admin/ai/whatsapp-insights?${params.toString()}`)
    },
    onSuccess: ({ whatsapp_insights }) => setWhatsAppInsightsReport(whatsapp_insights),
    onError: (error: Error) => toast.error("Could not load WhatsApp activity", { description: error.message }),
  })

  const addKnowledge = useMutation({
    mutationFn: () => sdk.client.fetch("/admin/ai/knowledge", {
      method: "POST",
      body: { title, category: category || null, content },
    }),
    onSuccess: async () => {
      setTitle("")
      setCategory("")
      setContent("")
      await queryClient.invalidateQueries({ queryKey: KNOWLEDGE_QUERY })
      toast.success("Knowledge entry added")
    },
    onError: (error: Error) => toast.error("Could not add entry", { description: error.message }),
  })

  const deleteKnowledge = useMutation({
    mutationFn: (id: string) => sdk.client.fetch(`/admin/ai/knowledge/${id}`, { method: "DELETE" }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: KNOWLEDGE_QUERY }),
    onError: (error: Error) => toast.error("Could not remove entry", { description: error.message }),
  })

  const save = (event: FormEvent) => {
    event.preventDefault()
    saveConfiguration.mutate()
  }

  return (
    <SingleColumnPage widgets={{ before: [], after: [] }}>
      <div className="flex flex-col gap-y-6">
        <div>
          <Heading level="h1">AI & Social Commerce</Heading>
          <Text className="text-ui-fg-subtle">Configure a low-cost assistant, connect Meta channels, and maintain the store policy knowledge it can use.</Text>
        </div>

        <Container className="flex flex-col gap-y-4">
          <Heading level="h2">WhatsApp consented activity</Heading>
          <Text className="text-ui-fg-subtle">Counts only processed inbound WhatsApp events from people who separately opted in to anonymous activity aggregation at checkout. Transactional order-update consent is separate. Phone numbers and sender identifiers are never returned. Coverage is always partial because this event ledger cannot prove complete webhook delivery; withdrawn and unconsented users are excluded.</Text>
          <div className="grid grid-cols-1 gap-3 md:grid-cols-4">
            <label className="flex flex-col gap-1 text-sm">Connected WhatsApp account
              <select className="h-9 rounded-md border px-2" value={whatsAppInsightsAccount} onChange={(event) => { setWhatsAppInsightsAccount(event.target.value); setWhatsAppInsightsReport(null) }}>
                <option value="">Select a connected account</option>
                {(configurations.data?.configurations || []).filter((item) => item.kind === "whatsapp" && item.enabled && item.configured).map((item) => (
                  <option key={item.id} value={item.account_id}>{item.account_id}</option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1 text-sm">Start date (UTC)
              <Input type="date" value={whatsAppInsightsStartDate} max={whatsAppInsightsEndDate} onChange={(event) => setWhatsAppInsightsStartDate(event.target.value)} />
            </label>
            <label className="flex flex-col gap-1 text-sm">End date (UTC)
              <Input type="date" value={whatsAppInsightsEndDate} min={whatsAppInsightsStartDate} max={new Date().toISOString().slice(0, 10)} onChange={(event) => setWhatsAppInsightsEndDate(event.target.value)} />
            </label>
          </div>
          <Button type="button" className="w-fit" isLoading={fetchWhatsAppInsights.isPending} disabled={!whatsAppInsightsAccount} onClick={() => fetchWhatsAppInsights.mutate()}>Fetch consented activity</Button>
          {whatsAppInsightsReport && <div className="grid gap-2 rounded-lg border p-3">
            <Text weight="plus">{whatsAppInsightsReport.account_id} · {whatsAppInsightsReport.coverage} coverage</Text>
            <Text className="text-ui-fg-subtle">[{whatsAppInsightsReport.window_started_on}, {whatsAppInsightsReport.window_ended_on}) · {whatsAppInsightsReport.window_timezone} · retrieved {new Date(whatsAppInsightsReport.ingested_at).toLocaleString()}</Text>
            {whatsAppInsightsReport.coverage === "partial" && <>
              <Text>{whatsAppInsightsReport.inbound_events.toLocaleString()} processed inbound messages</Text>
              <Text>{whatsAppInsightsReport.consented_conversations.toLocaleString()} consented sender conversations</Text>
            </>}
            {whatsAppInsightsReport.coverage === "unavailable" && <Text>No consented inbound message activity was available for this window.</Text>}
          </div>}
        </Container>

        <Container className="flex flex-col gap-y-4">
          <Heading level="h2">Provider and Meta credentials</Heading>
          <form className="grid grid-cols-1 gap-4 md:grid-cols-2" onSubmit={save}>
            <label className="flex flex-col gap-1 text-sm">Configuration type
              <select className="h-9 rounded-md border px-2" value={kind} onChange={(event) => {
                const next = event.target.value
                setKind(next)
                setAccountId(next === "llm" ? "default" : "")
              }}>
                <option value="llm">AI provider</option>
                <option value="whatsapp">WhatsApp Cloud API</option>
                <option value="messenger">Facebook Messenger</option>
                <option value="instagram">Instagram Messaging</option>
                <option value="meta_insights_facebook">Facebook Page insights (read-only)</option>
                <option value="meta_insights_instagram">Instagram insights (read-only)</option>
                <option value="search_console">Google Search Console (read-only)</option>
              </select>
            </label>
            {kind !== "llm" && <label className="flex flex-col gap-1 text-sm">{kind === "search_console" ? "Search Console property URL or sc-domain: property" : "Phone, page, or Instagram account ID"}
              <Input value={accountId} onChange={(event) => setAccountId(event.target.value)} required maxLength={kind === "search_console" ? 512 : 128} />
            </label>}
            {kind === "llm" ? <>
              <label className="flex flex-col gap-1 text-sm">Provider
                <select className="h-9 rounded-md border px-2" value={provider} onChange={(event) => setProvider(event.target.value)}>
                  <option value="deepseek">DeepSeek</option><option value="qwen">Qwen</option><option value="ollama">Ollama (local, no API token)</option>
                </select>
              </label>
              <label className="flex flex-col gap-1 text-sm">Model
                <Input value={model} maxLength={128} onChange={(event) => setModel(event.target.value)} required />
              </label>
              {provider !== "ollama" && <label className="flex flex-col gap-1 text-sm md:col-span-2">Provider API key
                <Input type="password" autoComplete="new-password" value={secrets.api_key} onChange={(event) => setSecrets({ ...secrets, api_key: event.target.value })} placeholder="Leave blank to keep the saved key" />
              </label>}
            </> : kind === "search_console" ? <>
              <Text className="text-ui-fg-subtle md:col-span-2">Connect with Google using read-only Search Console access. The OAuth client and callback are configured by the platform; credentials are encrypted after authorization.</Text>
              <Button type="button" className="w-fit md:col-span-2" isLoading={startSearchConsoleOAuth.isPending} disabled={!accountId.trim()} onClick={() => {
                setSearchConsoleProperty(accountId.trim())
                startSearchConsoleOAuth.mutate(accountId.trim())
              }}>Connect Google Search Console</Button>
            </> : kind.startsWith("meta_insights_") ? <>
              <Text className="text-ui-fg-subtle md:col-span-2">Use a dedicated read-only Meta token with the insights permissions for this account. Do not reuse a messaging token. Meta validates the granted permissions when insights are fetched.</Text>
              <label className="flex flex-col gap-1 text-sm">Meta app secret
                <Input type="password" autoComplete="new-password" value={secrets.app_secret} onChange={(event) => setSecrets({ ...secrets, app_secret: event.target.value })} placeholder="Leave blank to keep the saved value" />
              </label>
              <label className="flex flex-col gap-1 text-sm md:col-span-2">Read-only insights access token
                <Input type="password" autoComplete="new-password" value={secrets.access_token} onChange={(event) => setSecrets({ ...secrets, access_token: event.target.value })} placeholder="Leave blank to keep the saved token" />
              </label>
            </> : <>
              <label className="flex flex-col gap-1 text-sm">Meta app secret
                <Input type="password" autoComplete="new-password" value={secrets.app_secret} onChange={(event) => setSecrets({ ...secrets, app_secret: event.target.value })} placeholder="Leave blank to keep the saved value" />
              </label>
              <label className="flex flex-col gap-1 text-sm">Webhook verify token
                <Input type="password" autoComplete="new-password" value={secrets.verify_token} onChange={(event) => setSecrets({ ...secrets, verify_token: event.target.value })} placeholder="Leave blank to keep the saved value" />
              </label>
              <label className="flex flex-col gap-1 text-sm md:col-span-2">Channel access token
                <Input type="password" autoComplete="new-password" value={secrets.access_token} onChange={(event) => setSecrets({ ...secrets, access_token: event.target.value })} placeholder="Leave blank to keep the saved token" />
              </label>
            </>}
            {kind !== "search_console" && <label className="flex items-center gap-2 text-sm md:col-span-2">
              <input type="checkbox" checked={enabled} onChange={(event) => setEnabled(event.target.checked)} /> Enable this configuration
            </label>}
            {kind !== "search_console" && <Button type="submit" isLoading={saveConfiguration.isPending} className="w-fit">Save encrypted configuration</Button>}
          </form>
          <div className="grid gap-2">
            {(configurations.data?.configurations || []).map((item) => (
              <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border p-3" key={item.id}>
                <Text>{item.kind} · {item.account_id} · {item.enabled ? "Enabled" : "Disabled"} · version {item.version}</Text>
                {item.kind === "search_console" && <Button type="button" variant="secondary" size="small" isLoading={disconnectSearchConsole.isPending} onClick={() => disconnectSearchConsole.mutate(item.account_id)}>Disconnect</Button>}
                {item.kind === "meta_insights_facebook" && <Button type="button" variant="secondary" size="small" isLoading={disconnectMetaInsights.isPending} onClick={() => disconnectMetaInsights.mutate({ channel: "messenger", account_id: item.account_id })}>Disconnect insights</Button>}
                {item.kind === "meta_insights_instagram" && <Button type="button" variant="secondary" size="small" isLoading={disconnectMetaInsights.isPending} onClick={() => disconnectMetaInsights.mutate({ channel: "instagram", account_id: item.account_id })}>Disconnect insights</Button>}
                {(item.kind === "meta_insights_facebook" || item.kind === "meta_insights_instagram") && <Button type="button" variant="secondary" size="small" isLoading={setMetaInsightsEnabled.isPending} onClick={() => setMetaInsightsEnabled.mutate({ item, enabled: !item.enabled })}>{item.enabled ? "Disable insights" : "Enable insights"}</Button>}
                {item.webhook_id && <Text className="text-ui-fg-subtle">Webhook: /webhooks/meta/{item.webhook_id}</Text>}
              </div>
            ))}
          </div>
          <Text className="text-ui-fg-subtle">Credential values are write-only in this dashboard. Blank secret fields preserve the currently saved value.</Text>
        </Container>

        <Container className="flex flex-col gap-y-4">
          <Heading level="h2">Facebook and Instagram insights</Heading>
          <Text className="text-ui-fg-subtle">Read-only aggregates use a separate encrypted token from Messenger or Instagram messaging. No posts, messages, or audience demographics are collected. Required permissions: {metaInsightsChannel === "messenger" ? "pages_read_engagement and read_insights" : "instagram_basic and instagram_manage_insights"}. Meta checks the token’s permissions when fetched; missing permissions and incomplete pages are shown as errors or partial coverage.</Text>
          <div className="grid grid-cols-1 gap-3 md:grid-cols-4">
            <label className="flex flex-col gap-1 text-sm">Connected account type
              <select className="h-9 rounded-md border px-2" value={metaInsightsChannel} onChange={(event) => { setMetaInsightsChannel(event.target.value); setMetaInsightsAccount(""); setMetaInsightsReport(null) }}>
                <option value="messenger">Facebook Page</option>
                <option value="instagram">Instagram professional account</option>
              </select>
            </label>
            <label className="flex flex-col gap-1 text-sm">Account
              <select className="h-9 rounded-md border px-2" value={metaInsightsAccount} onChange={(event) => { setMetaInsightsAccount(event.target.value); setMetaInsightsReport(null) }}>
                <option value="">Select a connected account</option>
                {(configurations.data?.configurations || []).filter((item) => item.kind === (metaInsightsChannel === "messenger" ? "meta_insights_facebook" : "meta_insights_instagram") && item.enabled && item.configured).map((item) => (
                  <option key={item.id} value={item.account_id}>{item.account_id}</option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1 text-sm">Start date (UTC)
              <Input type="date" value={metaInsightsStartDate} max={metaInsightsEndDate} onChange={(event) => setMetaInsightsStartDate(event.target.value)} />
            </label>
            <label className="flex flex-col gap-1 text-sm">End date (UTC)
              <Input type="date" value={metaInsightsEndDate} min={metaInsightsStartDate} max={new Date().toISOString().slice(0, 10)} onChange={(event) => setMetaInsightsEndDate(event.target.value)} />
            </label>
          </div>
          <Button type="button" className="w-fit" isLoading={fetchMetaInsights.isPending} disabled={!metaInsightsAccount} onClick={() => fetchMetaInsights.mutate()}>Fetch account insights</Button>
          {metaInsightsReport && <div className="grid gap-2 rounded-lg border p-3">
            <Text weight="plus">{metaInsightsReport.account_id} · {metaInsightsReport.coverage} coverage</Text>
            <Text className="text-ui-fg-subtle">[{metaInsightsReport.window_started_on}, {metaInsightsReport.window_ended_on}) · {metaInsightsReport.window_timezone} · {metaInsightsReport.rows_received} metric rows · retrieved {new Date(metaInsightsReport.ingested_at).toLocaleString()}</Text>
            {metaInsightsReport.observations.map((metric) => (
              <div className="flex items-center justify-between gap-2 rounded-md bg-ui-bg-subtle p-2" key={metric.metric}>
                <Text>{metric.metric.replace(/_/g, " ")}</Text>
                <Text>{String(metric.value)} {metric.unit}</Text>
              </div>
            ))}
            {metaInsightsReport.coverage !== "complete" && <Text className="text-ui-fg-subtle">Meta returned incomplete or unavailable account metrics. Missing values are not inferred as zero.</Text>}
          </div>}
          {metaInsightsSnapshots.data && <div className="grid gap-2 rounded-lg border p-3">
            <Text weight="plus">Saved Meta insights history</Text>
            {metaInsightsSnapshots.data.snapshots.length === 0
              ? <Text className="text-ui-fg-subtle">No Meta insight snapshot has been fetched for this account.</Text>
              : metaInsightsSnapshots.data.snapshots.map((metric) => (
                <div className="flex items-center justify-between gap-2 rounded-md bg-ui-bg-subtle p-2" key={`${metric.metric}:${metric.window_started_on}:${metric.window_ended_on}`}>
                  <Text>{metric.metric.replace(/_/g, " ")} · {metric.coverage} · retrieved {new Date(metric.ingested_at).toLocaleString()}</Text>
                  <Text>{metric.value} {metric.unit} · [{metric.window_started_on}, {metric.window_ended_on})</Text>
                </div>
              ))}
          </div>}
          {metaInsightsSnapshots.data?.changes.length ? <div className="grid gap-2 rounded-lg border p-3">
            <Text weight="plus">Adjacent-window changes</Text>
            <Text className="text-ui-fg-subtle">Only complete, adjacent windows with matching account and metric definitions are compared.</Text>
            {metaInsightsSnapshots.data.changes.map((change) => (
              <Text key={`${change.metric}:${change.current_window_started_on}`}>
                {change.metric.replace(/_/g, " ")}: {change.previous_value} → {change.current_value} {change.unit} · {change.change_basis_points} basis points
              </Text>
            ))}
          </div> : null}
          {metaInsightsAudit.data?.history.length ? <div className="grid gap-2 rounded-lg border p-3">
            <Text weight="plus">Connection history</Text>
            {metaInsightsAudit.data.history.map((entry) => <Text key={`${entry.action}:${entry.created_at}`}>{entry.action} · {entry.actor_id} · {new Date(entry.created_at).toLocaleString()}</Text>)}
          </div> : null}
        </Container>

        <Container className="flex flex-col gap-y-4">
          <Heading level="h2">Google Search Console</Heading>
          <Text className="text-ui-fg-subtle">Read-only property data. Search Console describes your own search impressions, not broad keyword demand. Google's reporting dates use the Pacific calendar; omitted dates and incomplete results are shown as partial coverage.</Text>
          <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
            <label className="flex flex-col gap-1 text-sm">Configured property
              <select className="h-9 rounded-md border px-2" value={searchConsoleProperty} onChange={(event) => setSearchConsoleProperty(event.target.value)}>
                <option value="">Select a connected property</option>
                {(configurations.data?.configurations || []).filter((item) => item.kind === "search_console" && item.enabled && item.configured).map((item) => (
                  <option key={item.id} value={item.account_id}>{item.account_id}</option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1 text-sm">Start date (Pacific)
              <Input type="date" value={searchConsoleStartDate} max={searchConsoleEndDate} onChange={(event) => setSearchConsoleStartDate(event.target.value)} />
            </label>
            <label className="flex flex-col gap-1 text-sm">End date (Pacific)
              <Input type="date" value={searchConsoleEndDate} min={searchConsoleStartDate} max={new Date().toISOString().slice(0, 10)} onChange={(event) => setSearchConsoleEndDate(event.target.value)} />
            </label>
          </div>
          <Button type="button" className="w-fit" isLoading={fetchSearchConsole.isPending} disabled={!searchConsoleProperty} onClick={() => fetchSearchConsole.mutate()}>Fetch authorized search data</Button>
          {searchConsoleReport && <div className="grid gap-2 rounded-lg border p-3">
            <Text weight="plus">{searchConsoleReport.property} · {searchConsoleReport.coverage} coverage</Text>
            <Text className="text-ui-fg-subtle">[{searchConsoleReport.window_started_on}, {searchConsoleReport.window_ended_on}) · {searchConsoleReport.window_timezone} · {searchConsoleReport.rows_received} date rows · retrieved {new Date(searchConsoleReport.ingested_at).toLocaleString()}</Text>
            {searchConsoleReport.observations.map((metric) => (
              <div className="flex items-center justify-between gap-2 rounded-md bg-ui-bg-subtle p-2" key={metric.metric}>
                <Text>{metric.metric.replace(/_/g, " ")}</Text>
                <Text>{String(metric.value)} {metric.unit}{metric.sample_size === null ? "" : ` · ${metric.sample_size} date rows`}</Text>
              </div>
            ))}
            {searchConsoleReport.coverage !== "complete" && <Text className="text-ui-fg-subtle">The provider returned incomplete or unavailable data. No missing dates are treated as zero.</Text>}
          </div>}
          {searchConsoleSnapshots.data && <div className="grid gap-2 rounded-lg border p-3">
            <Text weight="plus">Saved Search Console history</Text>
            {searchConsoleSnapshots.data.snapshots.length === 0
              ? <Text className="text-ui-fg-subtle">No Search Console snapshot has been fetched for this property.</Text>
              : <>
                <Text className="text-ui-fg-subtle">Shows the latest 100 observations from the past 400 days. Expired rows are pruned during the next collection.</Text>
                {searchConsoleSnapshots.data.snapshots.map((metric) => (
                  <div className="flex items-center justify-between gap-2 rounded-md bg-ui-bg-subtle p-2" key={`${metric.metric}:${metric.window_started_on}:${metric.window_ended_on}`}>
                    <Text>{metric.metric.replace(/_/g, " ")} · {metric.coverage} · retrieved {new Date(metric.ingested_at).toLocaleString()}</Text>
                    <Text>{metric.value} {metric.unit} · [{metric.window_started_on}, {metric.window_ended_on})</Text>
                  </div>
                ))}
              </>}
          </div>}
          {searchConsoleSnapshots.data?.changes.length ? <div className="grid gap-2 rounded-lg border p-3">
            <Text weight="plus">Adjacent-window changes</Text>
            <Text className="text-ui-fg-subtle">Only complete, equal-length, adjacent windows with matching source definitions are compared.</Text>
            {searchConsoleSnapshots.data.changes.map((change) => (
              <Text key={`${change.metric}:${change.current_window_started_on}`}>
                {change.metric.replace(/_/g, " ")}: {change.previous_value} → {change.current_value} {change.unit} · {change.change_basis_points} basis points
              </Text>
            ))}
          </div> : null}
          {searchConsoleAudit.data?.history.length ? <div className="grid gap-2 rounded-lg border p-3">
            <Text weight="plus">Connection activity</Text>
            {searchConsoleAudit.data.history.map((event, index) => (
              <Text className="text-ui-fg-subtle" key={`${event.created_at}:${event.action}:${index}`}>
                {event.action} by {event.actor_id} · {new Date(event.created_at).toLocaleString()}
              </Text>
            ))}
          </div> : null}
        </Container>

        <Container className="flex flex-col gap-y-4">
          <Heading level="h2">Chat-only prices</Heading>
          <Text className="text-ui-fg-subtle">Set an exact final unit price for one published variant. The assistant can quote it only when the live website price is higher, the currency matches, and the offer is active. Offers never stack.</Text>
          <form className="grid grid-cols-1 gap-3 md:grid-cols-2" onSubmit={(event) => { event.preventDefault(); saveChatOffer.mutate() }}>
            <label className="flex flex-col gap-1 text-sm">Published variant ID
              <Input value={offerVariant} onChange={(event) => setOfferVariant(event.target.value)} maxLength={128} required />
            </label>
            <label className="flex flex-col gap-1 text-sm">Currency code
              <Input value={offerCurrency} onChange={(event) => setOfferCurrency(event.target.value.toUpperCase())} minLength={3} maxLength={3} required />
            </label>
            <label className="flex flex-col gap-1 text-sm">Exact chat unit price
              <Input inputMode="decimal" value={offerPrice} onChange={(event) => setOfferPrice(event.target.value)} placeholder="995.00" maxLength={32} required />
            </label>
            <label className="flex flex-col gap-1 text-sm">Maximum quantity per order
              <Input type="number" min={1} max={1000} step={1} value={offerMaxQuantity} onChange={(event) => setOfferMaxQuantity(event.target.value)} required />
            </label>
            <label className="flex flex-col gap-1 text-sm">Starts at (optional)
              <Input type="datetime-local" value={offerStartsAt} onChange={(event) => setOfferStartsAt(event.target.value)} />
            </label>
            <label className="flex flex-col gap-1 text-sm">Ends at (optional)
              <Input type="datetime-local" value={offerEndsAt} onChange={(event) => setOfferEndsAt(event.target.value)} />
            </label>
            <label className="flex items-center gap-2 text-sm md:col-span-2">
              <input type="checkbox" checked={offerEnabled} onChange={(event) => setOfferEnabled(event.target.checked)} />
              Enable this chat price {editingOfferId ? "after saving" : "when saving"}
            </label>
            <div className="flex gap-2 md:col-span-2">
              <Button type="submit" isLoading={saveChatOffer.isPending} className="w-fit">Save chat price</Button>
              {editingOfferId && <Button type="button" variant="secondary" onClick={cancelChatOfferEdit}>Cancel edit</Button>}
            </div>
          </form>
          <div className="grid gap-2">
            {(chatOffers.data?.chat_offers || []).map((offer) => (
              <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border p-3" key={offer.id}>
                <Text>{offer.variant_id} · {offer.chat_unit_price} {offer.currency_code} · max {offer.max_quantity} · {offer.enabled ? "Enabled" : "Disabled"}</Text>
                <div className="flex gap-2">
                  <Button variant="secondary" size="small" onClick={() => editChatOffer(offer)}>Edit</Button>
                  {offer.enabled && <Button variant="secondary" size="small" isLoading={deleteChatOffer.isPending} onClick={() => deleteChatOffer.mutate(offer.id)}>Disable</Button>}
                </div>
              </div>
            ))}
          </div>
          <Text className="text-ui-fg-subtle">Prices are decimal strings. Invalid, expired, stale, wrong-currency, or higher-than-live offers are ignored. Disabling an offer prevents new quotes; already-created drafts retain the quoted price for merchant review.</Text>
        </Container>

        <Container className="flex flex-col gap-y-4">
          <Heading level="h2">Store intelligence</Heading>
          <Text className="text-ui-fg-subtle">Token-free order trends compare the last 7 days with the previous 7 days. Buyer details are not included.</Text>
          {storeIntelligence.isPending && <Text>Loading order trends…</Text>}
          {storeIntelligence.isError && <Text className="text-ui-fg-error">Order trends could not be loaded.</Text>}
          {storeIntelligence.data && <>
            <Text className="text-ui-fg-subtle">Source: store orders · Generated {new Date(storeIntelligence.data.store_intelligence.generated_at).toLocaleString()}</Text>
            {storeIntelligence.data.store_intelligence.products.length === 0 ? <Text>No eligible product orders were found in either comparison window.</Text> : (
              <div className="grid gap-2">
                {storeIntelligence.data.store_intelligence.products.slice(0, 10).map((trend) => (
                  <div key={trend.product_id} className="flex flex-wrap items-center justify-between gap-2 rounded-lg border p-3">
                    <div>
                      <Text weight="plus">{trend.product_name}</Text>
                      <Text className="text-ui-fg-subtle">{trend.category || "Uncategorized"} · {trend.direction} · {trend.confidence} evidence volume</Text>
                    </div>
                    <Text>{trend.current_units} units in {trend.current_orders} orders · previous {trend.comparison_units}{trend.change_basis_points === null ? "" : ` · ${trend.change_basis_points >= 0 ? "+" : ""}${(trend.change_basis_points / 100).toFixed(1)}%`}</Text>
                  </div>
                ))}
              </div>
            )}
            {storeIntelligence.data.store_intelligence.products.length > 10 && <Text className="text-ui-fg-subtle">Showing the 10 highest-unit products from the report.</Text>}
            {storeIntelligence.data.store_intelligence.products_truncated && <Text className="text-ui-fg-subtle">The report is capped at 100 products; additional products are omitted.</Text>}
            {storeIntelligence.data.store_intelligence.categories.length > 0 && <div className="grid gap-2">
              <Heading level="h3">Category trends</Heading>
              {storeIntelligence.data.store_intelligence.categories.slice(0, 8).map((trend) => (
                <div key={trend.category} className="flex items-center justify-between gap-2 rounded-lg border p-3">
                  <div>
                    <Text weight="plus">{trend.category} · {trend.direction}</Text>
                    <Text className="text-ui-fg-subtle">Style suggestion: {trend.recommended_archetype} · {trend.recommended_image_aspect} images · rules {trend.archetype_rules_version} · {trend.archetype_basis === "category_unrecognized_fallback" ? "default for an unrecognized category" : "matched category words"}</Text>
                  </div>
                  <Text>{trend.current_units} units · previous {trend.comparison_units}</Text>
                </div>
              ))}
            </div>}
            {storeIntelligence.data.store_intelligence.categories.length > 8 && <Text className="text-ui-fg-subtle">Showing the eight highest-unit categories from the report.</Text>}
            {storeIntelligence.data.store_intelligence.categories_truncated && <Text className="text-ui-fg-subtle">The report is capped at 100 categories; additional categories are omitted.</Text>}
            <div className="grid gap-2">
              <Heading level="h3">Options to review</Heading>
              {storeIntelligence.data.store_intelligence.recommendations.length === 0 ? <Text className="text-ui-fg-subtle">No option met the minimum evidence threshold of five orders in the current window.</Text> : (
                storeIntelligence.data.store_intelligence.recommendations.map((option) => (
                  <div key={option.id} className="rounded-lg border p-3">
                    <Text weight="plus">{option.action.replace(/_/g, " ")}: {option.subject_label}</Text>
                    <Text className="text-ui-fg-subtle">{option.rationale}</Text>
                    <Text className="text-ui-fg-subtle">Evidence: {option.evidence} · {option.current_orders} orders · {option.confidence} evidence volume · owner review required</Text>
                    {option.review_decision && <Text className="text-ui-fg-subtle">Owner review: {option.review_decision}</Text>}
                    {option.knowledge_lookup === "matched" && <Text className="text-ui-fg-subtle">Related merchant knowledge: {option.knowledge_sources.map((source) => `${source.title} (${new Date(source.updated_at).toLocaleDateString()})`).join(" · ")}</Text>}
                    {option.knowledge_lookup === "no_match" && <Text className="text-ui-fg-subtle">No merchant-authored knowledge matched this option.</Text>}
                    {option.knowledge_lookup === "unavailable" && <Text className="text-ui-fg-subtle">Merchant knowledge lookup is unavailable; this option uses order evidence only.</Text>}
                    <div className="mt-3 flex flex-wrap gap-2">
                      <Button size="small" variant="secondary" isLoading={reviewRecommendation.isPending} onClick={() => reviewRecommendation.mutate({ id: option.id, decision: "approved", requestId: crypto.randomUUID() })}>Mark reviewed</Button>
                      <Button size="small" variant="secondary" isLoading={reviewRecommendation.isPending} onClick={() => reviewRecommendation.mutate({ id: option.id, decision: "dismissed", requestId: crypto.randomUUID() })}>Dismiss</Button>
                      {option.review_decision && <Button size="small" variant="secondary" isLoading={reviewRecommendation.isPending} onClick={() => reviewRecommendation.mutate({ id: option.id, decision: "reverted", requestId: crypto.randomUUID() })}>Revert review</Button>}
                      <Button size="small" variant="secondary" onClick={() => setReviewHistoryId((current) => current === option.id ? null : option.id)}>
                        {reviewHistoryId === option.id ? "Hide history" : "Review history"}
                      </Button>
                    </div>
                    {reviewHistoryId === option.id && <div className="mt-3 grid gap-2 rounded-md bg-ui-bg-subtle p-3">
                      <Text weight="plus">Append-only audit history</Text>
                      {reviewHistory.isPending && <Text className="text-ui-fg-subtle">Loading review history…</Text>}
                      {reviewHistory.isError && <Text className="text-ui-fg-error">Review history could not be loaded.</Text>}
                      {reviewHistory.data?.history.length === 0 && <Text className="text-ui-fg-subtle">No review events are recorded for this recommendation.</Text>}
                      {reviewHistory.data?.history.map((event) => (
                        <Text key={event.request_id} className="text-ui-fg-subtle">
                          {event.decision} · {new Date(event.created_at).toLocaleString()} · actor {event.actor_id} · {event.evidence_source} · {event.rule_version}
                        </Text>
                      ))}
                    </div>}
                  </div>
                ))
              )}
              {storeIntelligence.data.store_intelligence.recommendations_truncated && <Text className="text-ui-fg-subtle">Showing the ten highest-priority options; other options are omitted.</Text>}
            </div>
            <Text className="text-ui-fg-subtle">This report uses store orders only. Social-channel signals remain hidden until their adapters are available; authorized Search Console results are shown separately.</Text>
            <div className="mt-3 flex flex-col gap-2 rounded-lg border p-3">
              <Text weight="plus">Optional local model ranking</Text>
              <Text className="text-ui-fg-subtle">Runs only when requested and only against the fixed local Ollama endpoint. It reorders existing candidates, must cite their evidence, and cannot take actions or change prices. Uncertainty is a model label, not a statistical confidence measure.</Text>
              <Button variant="secondary" size="small" className="w-fit" isLoading={rankStoreOptions.isPending} onClick={() => rankStoreOptions.mutate()}>Rank with local model</Button>
              {modelRanking?.status === "ranked" && <ol className="list-decimal pl-5">
                {modelRanking.ranked.map((item) => {
                  const option = storeIntelligence.data?.store_intelligence.recommendations.find((candidate) => candidate.id === item.recommendation_id)
                  const citations = modelRanking.evidence_citations.filter((citation) => citation.recommendation_id === item.recommendation_id)
                  return option ? <li key={item.recommendation_id}><Text>{option.subject_label} · uncertainty {item.uncertainty} · cited {citations.map((citation) => citation.source_id).join(", ")}</Text></li> : null
                })}
              </ol>}
              {modelRanking?.status === "abstained" && <Text className="text-ui-fg-subtle">Local ranking abstained: {modelRanking.abstention_reason?.replace(/_/g, " ") || "insufficient evidence"}. The deterministic report remains available.</Text>}
            </div>
          </>}
        </Container>

        <Container className="flex flex-col gap-y-4">
          <Heading level="h2">Store knowledge</Heading>
          <Text className="text-ui-fg-subtle">Add policies, FAQs, sizing details, business hours, and delivery guidance. Do not add customer information.</Text>
          <form className="grid grid-cols-1 gap-3 md:grid-cols-2" onSubmit={(event) => { event.preventDefault(); addKnowledge.mutate() }}>
            <Input aria-label="Title" placeholder="Title" value={title} onChange={(event) => setTitle(event.target.value)} required maxLength={200} />
            <Input aria-label="Category" placeholder="Category (optional)" value={category} onChange={(event) => setCategory(event.target.value)} maxLength={64} />
            <textarea className="min-h-28 rounded-md border p-2 text-sm md:col-span-2" aria-label="Knowledge content" placeholder="Policy or answer" value={content} onChange={(event) => setContent(event.target.value)} required maxLength={20000} />
            <Button type="submit" isLoading={addKnowledge.isPending} className="w-fit">Add knowledge</Button>
          </form>
          <div className="grid gap-3">
            {(knowledge.data?.knowledge || []).map((entry) => (
              <article className="rounded-md border p-3" key={entry.id}>
                <div className="flex items-center justify-between gap-2"><Heading level="h3">{entry.title}</Heading><Button variant="secondary" size="small" onClick={() => deleteKnowledge.mutate(entry.id)}>Remove</Button></div>
                {entry.category && <Text className="text-ui-fg-subtle">{entry.category}</Text>}
                <Text className="mt-2 whitespace-pre-wrap">{entry.content}</Text>
              </article>
            ))}
          </div>
        </Container>
      </div>
    </SingleColumnPage>
  )
}
