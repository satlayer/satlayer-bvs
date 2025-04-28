import "./globals.css";

import { Layout, Navbar } from "nextra-theme-docs";
import { Banner, Head } from "nextra/components";
import { getPageMap } from "nextra/page-map";
import { SatLayerWordmark } from "./Icon";
import type { ReactNode } from "react";
import type { Metadata } from "next";

export const metadata: Metadata = {
  metadataBase: new URL("https://build.satlayer.xyz"),
  title: {
    default: "SatLayer Docs",
    template: "%s | SatLayer Docs",
  },
};
const banner = (
  <Banner storageKey="wip" dismissible={false}>
    🚧 Work in progress! This site is under construction.
  </Banner>
);
const navbar = (
  <Navbar
    logo={
      <div className="flex items-center space-x-2">
        <SatLayerWordmark className="x:text-slate-900 x:dark:text-slate-100 h-6 w-full" />
      </div>
    }
  />
);
const footer = <></>;

export default async function RootLayout({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  return (
    <html
      lang="en"
      dir="ltr"
      // https://github.com/pacocoursey/next-themes/blob/1b510445a37e7c2cddb359b5f29fe8dce9fd4855/next-themes/README.md#with-app
      suppressHydrationWarning
    >
      <Head></Head>
      <body>
        <Layout
          banner={banner}
          sidebar={{
            defaultMenuCollapseLevel: 2,
          }}
          navbar={navbar}
          pageMap={await getPageMap()}
          docsRepositoryBase="https://github.com/satlayer/satlayer-bvs/tree/main/docs"
          footer={footer}
        >
          {children}
        </Layout>
      </body>
    </html>
  );
}
