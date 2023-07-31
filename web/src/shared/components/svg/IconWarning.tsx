import type { SVGProps } from 'react';
const SvgIconWarning = (props: SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width={18}
    height={18}
    fill="none"
    viewBox="0 0 18 18"
    {...props}
  >
    <circle cx={9} cy={9} r={9} fill="#CB3F3F" />
    <path
      fill="#FCEEEE"
      d="M9.933 10.849H8.066l-.294-6.802h2.455l-.294 6.802Zm-2.229 2.119c0-.164.03-.317.092-.458.062-.142.15-.263.264-.366.114-.103.248-.183.403-.243.155-.059.328-.088.52-.088.19 0 .364.03.519.088.155.06.29.14.403.243.114.103.202.224.264.366.061.141.092.294.092.458 0 .164-.031.316-.092.458-.062.141-.15.263-.264.365a1.28 1.28 0 0 1-.403.243c-.155.06-.328.089-.52.089-.191 0-.364-.03-.52-.089a1.28 1.28 0 0 1-.402-.243 1.05 1.05 0 0 1-.264-.365 1.135 1.135 0 0 1-.092-.458Z"
    />
  </svg>
);
export default SvgIconWarning;
