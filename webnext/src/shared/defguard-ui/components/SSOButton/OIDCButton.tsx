import './style.scss';

import { useMemo } from 'react';

import googleImage from './assets/google.png';

type Props = {
  text: string;
  url: string;
  testId?: string;
};

export const OIDCButton = ({ url, text }: Props) => {
  const RenderIcon = useMemo(() => {
    const { hostname } = new URL(url);
    if (hostname === 'accounts.google.com') {
      return IconGoogle;
    }
    if (hostname === 'login.microsoftonline.com') {
      return IconMicrosoft;
    }

    return IconDefault;
  }, [url]);

  return (
    <a className="oidc-button-link" href={url} target="_blank" rel="noopener noreferrer">
      <button className="oidc-button">
        <RenderIcon />
        <span>{text}</span>
      </button>
    </a>
  );
};

const IconMicrosoft = () => {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
    >
      <path d="M9.63103 1.20728H1.20728V9.6311H9.63103V1.20728Z" fill="#F25022" />
      <path d="M18.7928 1.20728H10.369V9.6311H18.7928V1.20728Z" fill="#7FBA00" />
      <path d="M9.63103 10.3689H1.20728V18.7928H9.63103V10.3689Z" fill="#00A4EF" />
      <path d="M18.7928 10.3689H10.369V18.7928H18.7928V10.3689Z" fill="#FFB900" />
    </svg>
  );
};

const IconGoogle = () => {
  return <img src={googleImage} width={20} height={20} loading="lazy" />;
};

const IconDefault = () => {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
    >
      <path
        d="M18.44 14.35L12.18 8.09C12.68 6.14 12.14 4.1 10.7 2.66C9.63 1.59 8.2 1 6.68 1C5.16 1 3.74 1.59 2.67 2.66C1.59 3.74 1 5.16 1 6.68C1 8.2 1.59 9.62 2.66 10.7C4.1 12.14 6.14 12.68 8.09 12.18L9.04 13.13C9.19 13.28 9.4 13.36 9.62 13.35L10.88 13.27L10.84 14.6C10.84 14.8 10.91 15 11.05 15.15C11.19 15.3 11.38 15.38 11.59 15.38H12.89L12.69 16.31C12.64 16.56 12.71 16.82 12.89 17L14.11 18.22C14.25 18.36 14.44 18.44 14.64 18.44H17.91C18.32 18.44 18.66 18.1 18.66 17.69V14.91C18.66 14.71 18.58 14.52 18.44 14.38V14.35ZM17.16 16.91H14.95L14.24 16.21L14.55 14.78C14.6 14.56 14.55 14.33 14.4 14.15C14.26 13.97 14.04 13.87 13.82 13.87H12.37L12.41 12.48C12.41 12.27 12.33 12.06 12.18 11.92C12.03 11.77 11.83 11.7 11.61 11.71L9.86 11.83L8.84 10.81C8.64 10.61 8.33 10.54 8.06 10.63C6.53 11.16 4.87 10.78 3.73 9.64C2.94 8.85 2.51 7.8 2.51 6.68C2.51 5.56 2.95 4.51 3.74 3.72C4.53 2.93 5.58 2.49 6.7 2.49C7.82 2.49 8.87 2.93 9.66 3.72C10.8 4.86 11.18 6.52 10.65 8.05C10.56 8.32 10.62 8.62 10.83 8.83L17.19 15.19V16.91H17.16ZM7.2 5.18C7.74 5.72 7.74 6.6 7.2 7.14C6.66 7.68 5.78 7.68 5.24 7.14C4.7 6.6 4.7 5.72 5.24 5.18C5.78 4.64 6.66 4.64 7.2 5.18Z"
        fill="#3961DB"
      />
    </svg>
  );
};
