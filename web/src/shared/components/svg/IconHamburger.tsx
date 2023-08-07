import type { SVGProps } from 'react';
const SvgIconHamburger = (props: SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width={22}
    height={23}
    fill="none"
    viewBox="0 0 22 23"
    {...props}
  >
    <path
      fill="#0C8CE0"
      d="M17 6.997H9a1 1 0 0 0 0 2h8a1 1 0 1 0 0-2ZM13 10.997H9a1 1 0 1 0 0 2h4a1 1 0 1 0 0-2ZM6 11.997a1 1 0 1 0-2 0 1 1 0 0 0 2 0ZM6 15.997a1 1 0 1 0-2 0 1 1 0 0 0 2 0ZM6 7.997a1 1 0 1 0-2 0 1 1 0 0 0 2 0ZM13 14.997H9a1 1 0 1 0 0 2h4a1 1 0 1 0 0-2Z"
    />
  </svg>
);
export default SvgIconHamburger;
