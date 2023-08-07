import type { SVGProps } from 'react';
const SvgIconArrowSingleLarge = (props: SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width={22}
    height={22}
    fill="none"
    viewBox="0 0 22 22"
    {...props}
  >
    <mask
      id="a"
      width={22}
      height={22}
      x={0}
      y={0}
      maskUnits="userSpaceOnUse"
      style={{
        maskType: 'luminance',
      }}
    >
      <path fill="#fff" d="M22 0H0v22h22V0Z" />
    </mask>
    <g fill="#899CA8" mask="url(#a)">
      <path d="m9.05 16.293 4.243-4.243a1 1 0 1 0-1.414-1.414l-4.243 4.243a1 1 0 1 0 1.414 1.414Z" />
      <path d="M13.292 10.293 9.049 6.05a1 1 0 1 0-1.414 1.414l4.243 4.243a1 1 0 0 0 1.414-1.414Z" />
    </g>
  </svg>
);
export default SvgIconArrowSingleLarge;
