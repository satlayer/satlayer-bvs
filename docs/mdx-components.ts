import { useMDXComponents as getThemeComponents } from "nextra-theme-docs";
import { MDXComponents } from "nextra/mdx-components";

const themeComponents = getThemeComponents();

export function useMDXComponents(components: Readonly<MDXComponents>) {
  return {
    ...themeComponents,
    ...components,
  };
}
