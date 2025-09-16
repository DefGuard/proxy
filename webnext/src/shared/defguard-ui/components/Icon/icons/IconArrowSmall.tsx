import type { SVGProps } from 'react';
import { ThemeKey } from '../../../types';

export const IconArrowSmall = (props: SVGProps<SVGSVGElement>) => {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
      {...props}
    >
      <path
        fill-rule="evenodd"
        clip-rule="evenodd"
        d="M8.06189 4.95046L12.3719 9.27046C12.6642 9.56345 12.6639 10.0378 12.3713 10.3305L8.06127 14.6405L7.00061 13.5798L10.7809 9.79956L7 6.00989L8.06189 4.95046Z"
        fill="#7E8794"
        style={{
          fill: ThemeKey.FgMuted,
        }}
      />
    </svg>
  );
};
