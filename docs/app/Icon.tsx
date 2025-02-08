import { ReactElement, SVGProps } from "react";

export function SatLayerIcon(props: Readonly<SVGProps<SVGSVGElement>>): ReactElement {
  return (
    <svg viewBox="0 0 256 256" {...props}>
      <path
        d="M4.43571 99.6802L130.217 43L163.721 58.0975L75.0958 98.1222L88.2951 104.333L176.49 63.8509L256 99.6802L130.217 156.359L96.5335 141.18L187.694 99.6817L176.046 94.2473L83.9216 135.497L4.43571 99.6802Z"
        fill="#0E0F12"
      />
      <path
        d="M125.782 213.389L0 155.155L33.3866 139.627L125.782 182.332L218.177 139.627L250.788 155.932L125.782 213.389Z"
        fill="#0E0F12"
      />
    </svg>
  );
}
