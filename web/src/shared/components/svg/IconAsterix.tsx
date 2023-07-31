import type { SVGProps } from 'react';
const SvgIconAsterix = (props: SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width={22}
    height={22}
    fill="none"
    viewBox="0 0 22 22"
    {...props}
  >
    <path
      fill="#CBD3D8"
      d="m7.17 14.366 8.66-5a1 1 0 1 0-1-1.732l-8.66 5a1 1 0 0 0 1 1.732Z"
    />
    <path
      fill="#CBD3D8"
      d="m15.83 12.634-8.66-5a1 1 0 0 0-1 1.732l8.66 5a1 1 0 0 0 1-1.732Z"
    />
    <path fill="#CBD3D8" d="M10 6v10a1 1 0 1 0 2 0V6a1 1 0 1 0-2 0Z" />
  </svg>
);
export default SvgIconAsterix;
