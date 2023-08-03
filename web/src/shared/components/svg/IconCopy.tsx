import type { SVGProps } from 'react';
const SvgIconCopy = (props: SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width={22}
    height={22}
    fill="none"
    viewBox="0 0 22 22"
    {...props}
  >
    <path fill="#899CA8" d="M18 11V5a1 1 0 0 0-1-1h-6a1 1 0 1 0 0 2h5v5a1 1 0 1 0 2 0Z" />
    <path
      fill="#899CA8"
      fillRule="evenodd"
      d="M14 9v8a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V9a1 1 0 0 1 1-1h8a1 1 0 0 1 1 1Zm-8 7h6v-6H6v6Z"
      clipRule="evenodd"
    />
  </svg>
);
export default SvgIconCopy;
