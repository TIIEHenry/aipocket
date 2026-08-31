/** Experimental proxy panel packs — queries not yet validated on real leaks. */
export type ExperimentalProxyPackId =
  | "proxy_nezha"
  | "proxy_wings"
  | "proxy_mqpanel"
  | "proxy_node_uri"

export const ALL_EXPERIMENTAL_PROXY_PACK_IDS: readonly ExperimentalProxyPackId[] = [
  "proxy_nezha",
  "proxy_wings",
  "proxy_mqpanel",
  "proxy_node_uri",
] as const

export interface ExperimentalProxyPackOption {
  id: ExperimentalProxyPackId
  label: string
  hint: string
  /** Extraction pipeline incomplete — discovery queries only. */
  discoveryOnly?: boolean
}

export const EXPERIMENTAL_PROXY_PACK_OPTIONS: readonly ExperimentalProxyPackOption[] = [
  {
    id: "proxy_nezha",
    label: "哪吒 Nezha",
    hint: "监控面板 subscribe 路径泄露",
  },
  {
    id: "proxy_wings",
    label: "Wings",
    hint: "Wings 面板订阅页 banner / html",
  },
  {
    id: "proxy_mqpanel",
    label: "MQPanel",
    hint: "MQPanel 订阅相关页面指纹",
  },
  {
    id: "proxy_node_uri",
    label: "节点 URI",
    hint: "页面内 vmess://、trojan:// 等 URI 片段",
    discoveryOnly: true,
  },
] as const

const PACK_ID_SET = new Set<string>(ALL_EXPERIMENTAL_PROXY_PACK_IDS)

export function isExperimentalProxyPackId(value: string): value is ExperimentalProxyPackId {
  return PACK_ID_SET.has(value)
}

/** Parse comma-separated env (`PROXY_SUB_EXTRA_PACKS`); unknown ids ignored. */
export function parseExperimentalProxyPacks(csv: string | null | undefined): ExperimentalProxyPackId[] {
  if (!csv?.trim()) return []
  const seen = new Set<ExperimentalProxyPackId>()
  const out: ExperimentalProxyPackId[] = []
  for (const part of csv.split(",")) {
    const id = part.trim()
    if (!isExperimentalProxyPackId(id) || seen.has(id)) continue
    seen.add(id)
    out.push(id)
  }
  return out
}

export function serializeExperimentalProxyPacks(ids: readonly ExperimentalProxyPackId[]): string {
  const seen = new Set<ExperimentalProxyPackId>()
  const ordered: ExperimentalProxyPackId[] = []
  for (const id of ids) {
    if (!isExperimentalProxyPackId(id) || seen.has(id)) continue
    seen.add(id)
    ordered.push(id)
  }
  return ordered.join(",")
}

export function experimentalProxyPackSummary(ids: readonly ExperimentalProxyPackId[]): string {
  if (ids.length === 0) return "未启用实验 pack"
  const labels = EXPERIMENTAL_PROXY_PACK_OPTIONS.filter((opt) => ids.includes(opt.id)).map(
    (opt) => opt.label,
  )
  if (labels.length <= 2) return labels.join("、")
  return `${labels.slice(0, 2).join("、")} 等 ${labels.length} 个`
}
