import type { MetaRecord } from "nextra";

const meta: MetaRecord = {
  developer: {
    theme: {
      collapsed: false,
    },
  },
  examples: {
    title: "Solidity Examples",
  },
  /**
   * ```sh
   * ln -s ../../../../../contracts/docs/src/src/interface/ISLAYRegistryV2.sol/interface.ISLAYRegistryV2.md page.md
   * ln -s ../../../../../contracts/docs/src/src/interface/ISLAYRewardsV2.sol/interface.ISLAYRewardsV2.md page.md
   * ln -s ../../../../../contracts/docs/src/src/interface/ISLAYRouterSlashingV2.sol/interface.ISLAYRouterSlashingV2.md page.md
   * ln -s ../../../../../contracts/docs/src/src/interface/ISLAYRouterV2.sol/interface.ISLAYRouterV2.md page.md
   * ln -s ../../../../../contracts/docs/src/src/interface/ISLAYVaultFactoryV2.sol/interface.ISLAYVaultFactoryV2.md page.md
   * ln -s ../../../../../contracts/docs/src/src/interface/ISLAYVaultV2.sol/interface.ISLAYVaultV2.md page.md
   * ```
   */
  contracts: {
    title: "Contract Reference",
  },
  deployed: {},
};

export default meta;
