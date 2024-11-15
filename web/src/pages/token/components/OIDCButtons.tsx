/* eslint-disable max-len */
import { useI18nContext } from '../../../i18n/i18n-react';
import { Button } from '../../../shared/components/layout/Button/Button';
import {
  ButtonSize,
  ButtonStyleVariant,
} from '../../../shared/components/layout/Button/types';
import './style.scss';

export const OpenIdLoginButton = ({
  url,
  display_name,
}: {
  url: string;
  display_name?: string;
}) => {
  const { hostname } = new URL(url);

  if (hostname === 'accounts.google.com') {
    return <GoogleButton url={url} />;
  } else if (hostname === 'login.microsoftonline.com') {
    return <MicrosoftButton url={url} />;
  } else {
    return <CustomButton url={url} display_name={display_name} />;
  }
};

const GoogleButton = ({ url }: { url: string }) => {
  return (
    <button
      className="gsi-material-button"
      onClick={() => {
        window.location.assign(url);
      }}
      type="button"
    >
      <div className="gsi-material-button-state"></div>
      <div className="gsi-material-button-content-wrapper">
        <div className="gsi-material-button-icon">
          <svg
            version="1.1"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 48 48"
            xmlnsXlink="http://www.w3.org/1999/xlink"
            style={{
              display: 'block',
            }}
          >
            <path
              fill="#EA4335"
              d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"
            ></path>
            <path
              fill="#4285F4"
              d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"
            ></path>
            <path
              fill="#FBBC05"
              d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"
            ></path>
            <path
              fill="#34A853"
              d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"
            ></path>
            <path fill="none" d="M0 0h48v48H0z"></path>
          </svg>
        </div>
        <span className="gsi-material-button-contents">Sign in with Google</span>
        <span
          style={{
            display: 'none',
          }}
        >
          Sign in with Google
        </span>
      </div>
    </button>
  );
};

const CustomButton = ({ url, display_name }: { url: string; display_name?: string }) => {
  const { LL } = useI18nContext();
  return (
    <Button
      size={ButtonSize.LARGE}
      styleVariant={ButtonStyleVariant.PRIMARY}
      text={`${LL.pages.token.card.oidcButton()} ${
        display_name && display_name.length > 0 ? display_name : 'OIDC'
      }`}
      data-testid="login-oidc"
      onClick={() => {
        window.location.assign(url);
      }}
      type="button"
    />
  );
};

const MicrosoftButton = ({ url }: { url: string }) => {
  return (
    <button
      onClick={() => {
        window.location.assign(url);
      }}
      className="ms-button"
      type="button"
    >
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 215 41">
        <title>Sign in with Microsoft</title>
        <rect width="215" height="41" fill="#fff" />
        <path d="M214,1V40H1V1H214m1-1H0V41H215V0Z" fill="#8c8c8c" />
        <path
          d="M45.812,25.082V23.288a2.849,2.849,0,0,0,.576.4,4.5,4.5,0,0,0,.707.3,5.513,5.513,0,0,0,.747.187,3.965,3.965,0,0,0,.688.065,2.937,2.937,0,0,0,1.637-.365,1.2,1.2,0,0,0,.538-1.062,1.16,1.16,0,0,0-.179-.649,1.928,1.928,0,0,0-.5-.5,5.355,5.355,0,0,0-.757-.435q-.437-.209-.935-.436c-.356-.19-.687-.383-1-.578a4.358,4.358,0,0,1-.8-.648,2.728,2.728,0,0,1-.534-.8,2.6,2.6,0,0,1-.194-1.047,2.416,2.416,0,0,1,.333-1.285,2.811,2.811,0,0,1,.879-.9,4.026,4.026,0,0,1,1.242-.528,5.922,5.922,0,0,1,1.42-.172,5.715,5.715,0,0,1,2.4.374v1.721a3.832,3.832,0,0,0-2.3-.645,4.106,4.106,0,0,0-.773.074,2.348,2.348,0,0,0-.689.241,1.5,1.5,0,0,0-.494.433,1.054,1.054,0,0,0-.19.637,1.211,1.211,0,0,0,.146.608,1.551,1.551,0,0,0,.429.468,4.276,4.276,0,0,0,.688.414c.271.134.584.28.942.436q.547.285,1.036.6a4.881,4.881,0,0,1,.856.7,3.015,3.015,0,0,1,.586.846,2.464,2.464,0,0,1,.217,1.058,2.635,2.635,0,0,1-.322,1.348,2.608,2.608,0,0,1-.868.892,3.82,3.82,0,0,1-1.257.5,6.988,6.988,0,0,1-1.5.155c-.176,0-.392-.014-.649-.04s-.518-.067-.787-.117a7.772,7.772,0,0,1-.761-.187A2.4,2.4,0,0,1,45.812,25.082Z"
          fill="#5e5e5e"
        />
        <path
          d="M55.129,16.426a1.02,1.02,0,0,1-.714-.272.89.89,0,0,1-.3-.688.916.916,0,0,1,.3-.7,1.008,1.008,0,0,1,.714-.278,1.039,1.039,0,0,1,.732.278.909.909,0,0,1,.3.7.9.9,0,0,1-.3.678A1.034,1.034,0,0,1,55.129,16.426Zm.842,9.074h-1.7V18h1.7Z"
          fill="#5e5e5e"
        />
        <path
          d="M65.017,24.9q0,4.131-4.153,4.131a6.187,6.187,0,0,1-2.556-.491V26.986a4.726,4.726,0,0,0,2.337.7,2.342,2.342,0,0,0,2.672-2.628V24.24h-.029a2.947,2.947,0,0,1-4.742.436,4.041,4.041,0,0,1-.838-2.684,4.738,4.738,0,0,1,.9-3.04,3,3,0,0,1,2.476-1.128,2.384,2.384,0,0,1,2.2,1.216h.029V18h1.7Zm-1.684-2.835v-.973a1.91,1.91,0,0,0-.524-1.352A1.718,1.718,0,0,0,61.5,19.18a1.793,1.793,0,0,0-1.512.714,3.217,3.217,0,0,0-.546,2,2.774,2.774,0,0,0,.524,1.769,1.678,1.678,0,0,0,1.387.662,1.805,1.805,0,0,0,1.429-.632A2.391,2.391,0,0,0,63.333,22.064Z"
          fill="#5e5e5e"
        />
        <path
          d="M73.908,25.5h-1.7V21.273q0-2.1-1.486-2.1a1.622,1.622,0,0,0-1.282.582,2.162,2.162,0,0,0-.5,1.469V25.5H67.229V18h1.707v1.245h.029A2.673,2.673,0,0,1,71.4,17.824a2.265,2.265,0,0,1,1.868.795,3.57,3.57,0,0,1,.644,2.3Z"
          fill="#5e5e5e"
        />
        <path
          d="M80.962,16.426a1.02,1.02,0,0,1-.714-.272.89.89,0,0,1-.3-.688.916.916,0,0,1,.3-.7,1.008,1.008,0,0,1,.714-.278,1.039,1.039,0,0,1,.732.278.909.909,0,0,1,.3.7.9.9,0,0,1-.3.678A1.034,1.034,0,0,1,80.962,16.426ZM81.8,25.5H80.1V18h1.7Z"
          fill="#5e5e5e"
        />
        <path
          d="M90.7,25.5H89V21.273q0-2.1-1.486-2.1a1.622,1.622,0,0,0-1.282.582,2.157,2.157,0,0,0-.506,1.469V25.5H84.023V18H85.73v1.245h.03a2.673,2.673,0,0,1,2.431-1.421,2.265,2.265,0,0,1,1.868.795,3.57,3.57,0,0,1,.644,2.3Z"
          fill="#5e5e5e"
        />
        <path
          d="M106.984,18l-2.212,7.5h-1.78l-1.361-5.083a3.215,3.215,0,0,1-.1-.659H101.5a3.069,3.069,0,0,1-.131.644l-1.48,5.1H98.145L95.939,18H97.7l1.363,5.405a3.16,3.16,0,0,1,.087.645H99.2a3.384,3.384,0,0,1,.117-.659L100.832,18h1.6l1.347,5.428a3.732,3.732,0,0,1,.095.644h.052a3.387,3.387,0,0,1,.11-.644L105.365,18Z"
          fill="#5e5e5e"
        />
        <path
          d="M109.1,16.426a1.018,1.018,0,0,1-.714-.272.886.886,0,0,1-.3-.688.912.912,0,0,1,.3-.7,1.006,1.006,0,0,1,.714-.278,1.039,1.039,0,0,1,.732.278.912.912,0,0,1,.3.7.9.9,0,0,1-.3.678A1.034,1.034,0,0,1,109.1,16.426Zm.841,9.074h-1.7V18h1.7Z"
          fill="#5e5e5e"
        />
        <path
          d="M116.117,25.42a2.955,2.955,0,0,1-1.31.248q-2.182,0-2.183-2.094V19.333h-1.253V18h1.253V16.264l1.7-.483V18h1.794v1.333h-1.794v3.75a1.484,1.484,0,0,0,.241.952,1.006,1.006,0,0,0,.807.285,1.167,1.167,0,0,0,.746-.248Z"
          fill="#5e5e5e"
        />
        <path
          d="M124.248,25.5h-1.7V21.4q0-2.226-1.487-2.226a1.556,1.556,0,0,0-1.26.644,2.568,2.568,0,0,0-.513,1.649V25.5h-1.707V14.4h1.707v4.849h.029a2.685,2.685,0,0,1,2.432-1.421q2.5,0,2.5,3.055Z"
          fill="#5e5e5e"
        />
        <path
          d="M141.907,25.5h-1.728V18.7q0-.835.1-2.043h-.029a6.992,6.992,0,0,1-.285.988L136.831,25.5h-1.2l-3.143-7.793a7.236,7.236,0,0,1-.277-1.047h-.029q.057.63.058,2.058V25.5h-1.611V15h2.453l2.762,7a10.884,10.884,0,0,1,.41,1.2h.036c.181-.551.327-.962.44-1.23L139.541,15h2.366Z"
          fill="#5e5e5e"
        />
        <path
          d="M145.158,16.426a1.022,1.022,0,0,1-.714-.272.89.89,0,0,1-.3-.688.916.916,0,0,1,.3-.7,1.009,1.009,0,0,1,.714-.278,1.043,1.043,0,0,1,.733.278.911.911,0,0,1,.3.7.9.9,0,0,1-.3.678A1.038,1.038,0,0,1,145.158,16.426ZM146,25.5h-1.7V18H146Z"
          fill="#5e5e5e"
        />
        <path
          d="M153.589,25.156a4.2,4.2,0,0,1-2.131.52,3.606,3.606,0,0,1-2.695-1.044,3.691,3.691,0,0,1-1.026-2.706,4.07,4.07,0,0,1,1.1-2.978,3.944,3.944,0,0,1,2.948-1.124,4.3,4.3,0,0,1,1.81.36v1.582a2.743,2.743,0,0,0-1.67-.586,2.32,2.32,0,0,0-1.766.728,2.665,2.665,0,0,0-.688,1.908,2.536,2.536,0,0,0,.648,1.838,2.3,2.3,0,0,0,1.739.674,2.716,2.716,0,0,0,1.729-.652Z"
          fill="#5e5e5e"
        />
        <path
          d="M159.625,19.619a1.4,1.4,0,0,0-.887-.242,1.513,1.513,0,0,0-1.259.682,3.04,3.04,0,0,0-.506,1.852V25.5h-1.7V18h1.7v1.545H157a2.606,2.606,0,0,1,.766-1.233,1.724,1.724,0,0,1,1.154-.444,1.432,1.432,0,0,1,.7.14Z"
          fill="#5e5e5e"
        />
        <path
          d="M164.02,25.676a3.719,3.719,0,0,1-2.773-1.051,3.8,3.8,0,0,1-1.036-2.787,3.7,3.7,0,0,1,3.991-4.014,3.6,3.6,0,0,1,2.739,1.033,3.986,3.986,0,0,1,.982,2.864,3.932,3.932,0,0,1-1.059,2.875A3.8,3.8,0,0,1,164.02,25.676Zm.08-6.5a1.938,1.938,0,0,0-1.575.7,2.913,2.913,0,0,0-.579,1.919,2.744,2.744,0,0,0,.586,1.856,1.965,1.965,0,0,0,1.568.678,1.87,1.87,0,0,0,1.542-.666,2.956,2.956,0,0,0,.538-1.9,3,3,0,0,0-.538-1.911A1.858,1.858,0,0,0,164.1,19.18Z"
          fill="#5e5e5e"
        />
        <path
          d="M169.182,25.266V23.691a3.392,3.392,0,0,0,2.1.725q1.539,0,1.538-.908a.714.714,0,0,0-.132-.436,1.241,1.241,0,0,0-.355-.318,2.784,2.784,0,0,0-.527-.25q-.3-.108-.677-.248a7.052,7.052,0,0,1-.832-.389,2.545,2.545,0,0,1-.615-.465,1.745,1.745,0,0,1-.37-.59,2.145,2.145,0,0,1-.125-.769,1.775,1.775,0,0,1,.256-.955,2.223,2.223,0,0,1,.69-.7,3.289,3.289,0,0,1,.98-.425,4.511,4.511,0,0,1,1.136-.143,5.181,5.181,0,0,1,1.86.315v1.487a3.136,3.136,0,0,0-1.816-.542,2.317,2.317,0,0,0-.582.066,1.472,1.472,0,0,0-.443.183.886.886,0,0,0-.286.282.669.669,0,0,0-.1.363.77.77,0,0,0,.1.41.93.93,0,0,0,.3.3,2.654,2.654,0,0,0,.483.234q.282.105.649.23a9.4,9.4,0,0,1,.867.4,2.886,2.886,0,0,1,.656.465,1.806,1.806,0,0,1,.417.6,2.034,2.034,0,0,1,.147.81,1.847,1.847,0,0,1-.264,1,2.205,2.205,0,0,1-.7.7,3.292,3.292,0,0,1-1.015.413,5.222,5.222,0,0,1-1.212.136A5.115,5.115,0,0,1,169.182,25.266Z"
          fill="#5e5e5e"
        />
        <path
          d="M179.443,25.676a3.717,3.717,0,0,1-2.772-1.051,3.793,3.793,0,0,1-1.036-2.787,3.7,3.7,0,0,1,3.991-4.014,3.6,3.6,0,0,1,2.739,1.033,3.986,3.986,0,0,1,.982,2.864,3.932,3.932,0,0,1-1.059,2.875A3.8,3.8,0,0,1,179.443,25.676Zm.08-6.5a1.936,1.936,0,0,0-1.574.7,2.908,2.908,0,0,0-.579,1.919,2.739,2.739,0,0,0,.586,1.856,1.964,1.964,0,0,0,1.567.678,1.868,1.868,0,0,0,1.542-.666,2.95,2.95,0,0,0,.539-1.9,2.99,2.99,0,0,0-.539-1.911A1.857,1.857,0,0,0,179.523,19.18Z"
          fill="#5e5e5e"
        />
        <path
          d="M189.067,15.781a1.533,1.533,0,0,0-.784-.2q-1.237,0-1.237,1.4V18h1.743v1.333h-1.736V25.5h-1.7V19.333h-1.282V18h1.282V16.784a2.362,2.362,0,0,1,.777-1.871,2.82,2.82,0,0,1,1.94-.684,2.879,2.879,0,0,1,1,.138Z"
          fill="#5e5e5e"
        />
        <path
          d="M194.23,25.42a2.955,2.955,0,0,1-1.31.248q-2.182,0-2.183-2.094V19.333h-1.253V18h1.253V16.264l1.7-.483V18h1.793v1.333h-1.793v3.75a1.484,1.484,0,0,0,.241.952,1,1,0,0,0,.806.285,1.165,1.165,0,0,0,.746-.248Z"
          fill="#5e5e5e"
        />
        <rect x="13" y="11" width="9" height="9" fill="#f25022" />
        <rect x="13" y="21" width="9" height="9" fill="#00a4ef" />
        <rect x="23" y="11" width="9" height="9" fill="#7fba00" />
        <rect x="23" y="21" width="9" height="9" fill="#ffb900" />
      </svg>
    </button>
  );
};
