import { Layout, Navbar } from "nextra-theme-docs";
import { Banner, Head } from "nextra/components";
import { getPageMap } from "nextra/page-map";
import "nextra-theme-docs/style.css";
import { ReactNode } from "react";
import { Metadata } from "next";
import { SatLayerIcon } from "./Icon";
import "./globals.css";

export const metadata: Metadata = {
  metadataBase: new URL("https://build.satlayer.xyz"),
  title: {
    default: "SatLayer Bitcoin Validated Service",
    template: "%s - SatLayer",
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
        <SatLayerIcon width="24px" height="24px" />
        <b>SatLayer</b>
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
