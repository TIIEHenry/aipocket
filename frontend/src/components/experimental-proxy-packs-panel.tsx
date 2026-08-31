import { useEffect, useId, useState } from "react"
import { Check, ChevronDown, FlaskConical } from "lucide-react"

import { cn } from "@/lib/utils"
import {
  ALL_EXPERIMENTAL_PROXY_PACK_IDS,
  EXPERIMENTAL_PROXY_PACK_OPTIONS,
  experimentalProxyPackSummary,
  type ExperimentalProxyPackId,
} from "@/lib/experimental-proxy-packs"

const SCAN_EXPANDED_KEY = "aipocket.scan.experimental-packs-expanded"

function readScanExpanded(): boolean {
  try {
    return window.localStorage.getItem(SCAN_EXPANDED_KEY) === "1"
  } catch {
    return false
  }
}

function writeScanExpanded(expanded: boolean) {
  try {
    window.localStorage.setItem(SCAN_EXPANDED_KEY, expanded ? "1" : "0")
  } catch {
    // ignore quota / private mode
  }
}

export interface ExperimentalProxyPacksPanelProps {
  value: readonly ExperimentalProxyPackId[]
  onChange: (next: ExperimentalProxyPackId[]) => void
  disabled?: boolean
  /** Scan page: compact chip row inside a callout; settings: full card grid */
  variant?: "compact" | "settings"
  /** Scan page: collapse to one-line summary; expand on click */
  collapsible?: boolean
  /** Scan page: local selection differs from saved settings */
  pendingSync?: boolean
  className?: string
}

function PackToggle({
  pack,
  checked,
  disabled,
  onToggle,
  layout,
}: Readonly<{
  pack: (typeof EXPERIMENTAL_PROXY_PACK_OPTIONS)[number]
  checked: boolean
  disabled: boolean
  onToggle: () => void
  layout: "compact" | "settings"
}>) {
  const checkBox = (
    <span
      className={cn(
        "flex shrink-0 items-center justify-center rounded-[4px] border transition-colors",
        layout === "compact" ? "size-4" : "mt-0.5 size-4",
        checked
          ? "border-accent bg-accent text-accent-text"
          : "border-border-primary bg-surface-base",
      )}
    >
      {checked ? <Check className="size-3" /> : null}
    </span>
  )

  if (layout === "compact") {
    return (
      <button
        type="button"
        disabled={disabled}
        aria-pressed={checked}
        title={pack.hint}
        onClick={onToggle}
        className={cn(
          "inline-flex max-w-full items-center gap-2 rounded-[4px] border px-3 py-[7px] text-left text-[13px] transition-colors",
          checked
            ? "border-accent bg-accent-dim font-semibold text-accent"
            : "border-border-primary bg-surface-raised text-text-secondary hover:text-text-primary",
          disabled && "cursor-not-allowed opacity-50",
        )}
      >
        {checkBox}
        <span className="truncate">{pack.label}</span>
        {pack.discoveryOnly ? (
          <span className="rounded bg-surface-inset px-1.5 py-0.5 font-mono text-[10px] text-text-muted">
            仅发现
          </span>
        ) : null}
      </button>
    )
  }

  return (
    <button
      type="button"
      disabled={disabled}
      aria-pressed={checked}
      onClick={onToggle}
      className={cn(
        "flex w-full items-start gap-2.5 rounded-md border px-3 py-2.5 text-left transition-colors",
        checked
          ? "border-accent/60 bg-accent-dim/40"
          : "border-border-primary bg-surface-inset hover:border-border-secondary",
        disabled && "cursor-not-allowed opacity-50",
      )}
    >
      {checkBox}
      <span className="min-w-0 flex-1">
        <span className="flex flex-wrap items-center gap-2">
          <span className="text-[13px] font-medium text-text-primary">{pack.label}</span>
          {pack.discoveryOnly ? (
            <span className="rounded bg-surface-overlay px-1.5 py-0.5 font-mono text-[10px] text-text-muted">
              仅 discovery
            </span>
          ) : null}
        </span>
        <span className="mt-1 block text-[12px] leading-relaxed text-text-secondary">{pack.hint}</span>
      </span>
    </button>
  )
}

export function ExperimentalProxyPacksPanel({
  value,
  onChange,
  disabled = false,
  variant = "settings",
  collapsible = false,
  pendingSync = false,
  className,
}: Readonly<ExperimentalProxyPacksPanelProps>) {
  const panelId = useId()
  const [expanded, setExpanded] = useState(() => (collapsible ? readScanExpanded() : true))

  const selected = new Set(value)
  const allSelected = ALL_EXPERIMENTAL_PROXY_PACK_IDS.every((id) => selected.has(id))
  const noneSelected = value.length === 0
  const isCompact = variant === "compact"

  useEffect(() => {
    if (collapsible && pendingSync) setExpanded(true)
  }, [collapsible, pendingSync])

  const setExpandedPersisted = (next: boolean) => {
    setExpanded(next)
    if (collapsible) writeScanExpanded(next)
  }

  const toggleOne = (id: ExperimentalProxyPackId) => {
    if (disabled) return
    onChange(selected.has(id) ? value.filter((item) => item !== id) : [...value, id])
  }

  const toggleAll = () => {
    if (disabled) return
    onChange(allSelected ? [] : [...ALL_EXPERIMENTAL_PROXY_PACK_IDS])
  }

  const badges = (
    <>
      <span className="rounded-full bg-warning/15 px-2 py-0.5 font-mono text-[10px] text-warning">
        实验
      </span>
      {pendingSync ? (
        <span className="rounded-full bg-accent/15 px-2 py-0.5 font-mono text-[10px] text-accent">
          待同步
        </span>
      ) : null}
      {value.length > 0 ? (
        <span className="rounded-full bg-surface-overlay px-2 py-0.5 font-mono text-[10px] text-text-muted">
          {value.length}/{ALL_EXPERIMENTAL_PROXY_PACK_IDS.length}
        </span>
      ) : null}
    </>
  )

  const collapsedSummary = (
    <p className="min-w-0 truncate font-mono text-[11px] text-text-muted">
      {experimentalProxyPackSummary(value)}
    </p>
  )

  const headerTitle = (
    <span className="inline-flex shrink-0 items-center gap-1.5 text-[13px] font-medium text-text-primary">
      <FlaskConical className="size-4 text-warning" />
      待取证 pack
    </span>
  )

  const settingsHeader = (
    <div className="flex flex-wrap items-start justify-between gap-3">
      <div className="min-w-0 flex-1 space-y-1">
        <div className="flex flex-wrap items-center gap-2">
          {headerTitle}
          <span className="rounded-full bg-warning/15 px-2 py-0.5 font-mono text-[10px] text-warning">
            实验 · 默认全关
          </span>
          {pendingSync ? (
            <span className="rounded-full bg-accent/15 px-2 py-0.5 font-mono text-[10px] text-accent">
              扫描前将同步到设置
            </span>
          ) : null}
        </div>
        <p className="font-mono text-[11px] leading-relaxed text-text-muted">
          查询尚未用真实泄露样本充分验证；与 PROXY_PACKS 合并后追加到 FOFA/Shodan/GitHub。
        </p>
      </div>
      <button
        type="button"
        disabled={disabled}
        onClick={toggleAll}
        className={cn(
          "shrink-0 rounded-[4px] border px-3 py-1.5 text-[12px] transition-colors",
          allSelected
            ? "border-accent bg-accent-dim font-semibold text-accent"
            : "border-border-primary bg-surface-raised text-text-secondary hover:text-text-primary",
          disabled && "cursor-not-allowed opacity-50",
        )}
      >
        {allSelected ? "全部取消" : "全部启用"}
      </button>
    </div>
  )

  const compactHeader = collapsible ? (
    <button
      type="button"
      aria-expanded={expanded}
      aria-controls={panelId}
      onClick={() => setExpandedPersisted(!expanded)}
      className="flex w-full min-w-0 items-center gap-3 text-left transition-colors hover:opacity-90"
    >
      <div className="flex min-w-0 flex-1 flex-col gap-1 sm:flex-row sm:items-center sm:gap-3">
        <div className="flex flex-wrap items-center gap-2">
          {headerTitle}
          {badges}
        </div>
        {!expanded ? collapsedSummary : null}
      </div>
      <ChevronDown
        className={cn(
          "size-4 shrink-0 text-text-muted transition-transform",
          expanded && "rotate-180",
        )}
      />
    </button>
  ) : (
    <div className="flex flex-wrap items-start justify-between gap-3">
      <div className="min-w-0 flex-1 space-y-1">
        <div className="flex flex-wrap items-center gap-2">
          {headerTitle}
          {badges}
        </div>
        <p className="font-mono text-[11px] leading-relaxed text-text-muted">
          追加到基础机场 pack · {experimentalProxyPackSummary(value)}
        </p>
      </div>
      <button
        type="button"
        disabled={disabled}
        onClick={toggleAll}
        className={cn(
          "shrink-0 rounded-[4px] border px-3 py-1.5 text-[12px] transition-colors",
          allSelected
            ? "border-accent bg-accent-dim font-semibold text-accent"
            : "border-border-primary bg-surface-raised text-text-secondary hover:text-text-primary",
          disabled && "cursor-not-allowed opacity-50",
        )}
      >
        {allSelected ? "全部取消" : "全部启用"}
      </button>
    </div>
  )

  const packList = isCompact ? (
    <div className="flex flex-wrap gap-2">
      {EXPERIMENTAL_PROXY_PACK_OPTIONS.map((pack) => (
        <PackToggle
          key={pack.id}
          pack={pack}
          checked={selected.has(pack.id)}
          disabled={disabled}
          onToggle={() => toggleOne(pack.id)}
          layout="compact"
        />
      ))}
    </div>
  ) : (
    <div className="grid gap-2 sm:grid-cols-2">
      {EXPERIMENTAL_PROXY_PACK_OPTIONS.map((pack) => (
        <PackToggle
          key={pack.id}
          pack={pack}
          checked={selected.has(pack.id)}
          disabled={disabled}
          onToggle={() => toggleOne(pack.id)}
          layout="settings"
        />
      ))}
    </div>
  )

  const expandedBody = (
    <>
      {isCompact && collapsible ? (
        <div className="flex flex-wrap items-center justify-between gap-2 border-t border-warning/20 pt-3">
          <p className="font-mono text-[11px] text-text-muted">
            追加到基础机场 pack · 默认全关
          </p>
          <button
            type="button"
            disabled={disabled}
            onClick={toggleAll}
            className={cn(
              "shrink-0 rounded-[4px] border px-3 py-1.5 text-[12px] transition-colors",
              allSelected
                ? "border-accent bg-accent-dim font-semibold text-accent"
                : "border-border-primary bg-surface-raised text-text-secondary hover:text-text-primary",
              disabled && "cursor-not-allowed opacity-50",
            )}
          >
            {allSelected ? "全部取消" : "全部启用"}
          </button>
        </div>
      ) : null}
      {packList}
      <p className="font-mono text-[11px] text-text-muted">
        {noneSelected
          ? "未启用任何实验 pack，仅使用基础 v2board / sspanel 等 pack。"
          : `已选 ${value.length} / ${ALL_EXPERIMENTAL_PROXY_PACK_IDS.length}`}
        {collapsible && pendingSync ? " · 开始扫描时将写入设置" : null}
      </p>
    </>
  )

  const showBody = !collapsible || expanded

  return (
    <div
      className={cn(
        "rounded-md border border-warning/25 bg-warning-dim/15 p-3 sm:p-4",
        disabled && "opacity-60",
        showBody ? "flex flex-col gap-3" : undefined,
        className,
      )}
    >
      {isCompact ? compactHeader : settingsHeader}
      {showBody ? (
        <div id={panelId} className="flex flex-col gap-3">
          {expandedBody}
        </div>
      ) : null}
    </div>
  )
}
