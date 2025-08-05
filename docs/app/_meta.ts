import type { MetaRecord } from "nextra";

const meta: MetaRecord = {
  _I: {
    type: "separator",
    title: "Introduction",
  },
  index: {},
  introduction: {
    display: "children",
  },
  _G: {
    type: "separator",
    title: "Getting Started",
  },
  "getting-started": {
    display: "children",
  },
  _D: {
    type: "separator",
    title: "BVS Developers",
  },
  developers: {
    display: "children",
  },
  _CW: {
    type: "separator",
    title: "CosmWasm (Rust)",
  },
  contracts: {},
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
