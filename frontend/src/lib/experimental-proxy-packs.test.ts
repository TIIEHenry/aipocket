import { describe, expect, it } from "vitest"

import {
  experimentalProxyPackSummary,
  parseExperimentalProxyPacks,
  serializeExperimentalProxyPacks,
} from "@/lib/experimental-proxy-packs"

describe("experimental-proxy-packs", () => {
  it("parses csv and ignores unknown ids", () => {
    expect(parseExperimentalProxyPacks("proxy_nezha, proxy_unknown, proxy_wings")).toEqual([
      "proxy_nezha",
      "proxy_wings",
    ])
  })

  it("serializes stable comma list", () => {
    expect(serializeExperimentalProxyPacks(["proxy_wings", "proxy_nezha"])).toBe(
      "proxy_wings,proxy_nezha",
    )
  })

  it("summarizes selection for scan UI", () => {
    expect(experimentalProxyPackSummary([])).toBe("未启用实验 pack")
    expect(experimentalProxyPackSummary(["proxy_nezha", "proxy_wings"])).toBe("哪吒 Nezha、Wings")
    expect(
      experimentalProxyPackSummary(["proxy_nezha", "proxy_wings", "proxy_mqpanel"]),
    ).toBe("哪吒 Nezha、Wings 等 3 个")
  })
})
