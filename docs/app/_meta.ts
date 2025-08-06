import type { MetaRecord } from "nextra";

const meta: MetaRecord = {
  _I: {
    type: "separator",
    title: "Introduction",
  },
  index: {},
  _G: {
    type: "separator",
    title: "Architecture",
  },
  architecture: {
    display: "children",
  },
  _EVM: {
    type: "separator",
    title: "EVM (Solidity)",
  },
  evm: {
    display: "children",
  },
  _CW: {
    type: "separator",
    title: "CosmWasm (Rust)",
  },
  cosmwasm: {
    display: "children",
  },
  _A: {
    type: "separator",
    title: "Protocol-agnostic",
  },
  agnostic: {
    display: "children",
  },
  _M: {
    type: "separator",
    title: "More",
  },
  "restaking-doc": {
    title: "Restaking Docs",
    type: "doc",
    href: "https://docs.satlayer.xyz/",
  },
  "restaking-app": {
    title: "Restaking App",
    type: "doc",
    href: "https://app.satlayer.xyz/",
  },
  "audited-by": {},
};

export default meta;
