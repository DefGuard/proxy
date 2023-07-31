import type { SVGProps } from 'react';
const SvgIconInfo = (props: SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width={18}
    height={18}
    fill="none"
    viewBox="0 0 18 18"
    {...props}
  >
    <circle cx={9} cy={9} r={9} fill="#899CA8" />
    <path
      fill="#F9F9F9"
      d="M8 8a1 1 0 0 1 2 0v5a1 1 0 1 1-2 0V8ZM8 5a1 1 0 1 1 2 0 1 1 0 0 1-2 0Z"
    />
  </svg>
);
export default SvgIconInfo;
