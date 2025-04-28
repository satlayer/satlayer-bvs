import type { MetaRecord } from "nextra";

const meta: MetaRecord = {
  _G: {
    type: "separator",
    title: "Getting Started",
  },
  index: {},
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
