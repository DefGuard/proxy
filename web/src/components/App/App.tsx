import '../../shared/scss/index.scss';
import 'dayjs/locale/en';

import dayjs from 'dayjs';
import duration from 'dayjs/plugin/duration';
import utc from 'dayjs/plugin/utc';
import { useEffect, useState } from 'react';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import { localStorageDetector } from 'typesafe-i18n/detectors';

import TypesafeI18n from '../../i18n/i18n-react';
import { detectLocale } from '../../i18n/i18n-util';
import { loadLocaleAsync } from '../../i18n/i18n-util.async';
import { EnrollmentPage } from '../../pages/enrollment/EnrollmentPage';
import { MainPage } from '../../pages/main/MainPage';
import { SessionTimeoutPage } from '../../pages/sessionTimeout/SessionTimeoutPage';
import { TokenPage } from '../../pages/token/TokenPage';
import { routes } from '../../shared/routes';

dayjs.extend(duration);
dayjs.extend(utc);

const router = createBrowserRouter([
  {
    path: routes.main,
    element: <MainPage />,
  },
  {
    path: routes.token,
    element: <TokenPage />,
  },
  {
    path: routes.timeout,
    element: <SessionTimeoutPage />,
  },
  {
    path: routes.enrollment,
    element: <EnrollmentPage />,
  },
]);

const detectedLocale = detectLocale(localStorageDetector);

export const App = () => {
  const [wasLoaded, setWasLoaded] = useState(false);

  useEffect(() => {
    loadLocaleAsync(detectedLocale).then(() => setWasLoaded(true));
    dayjs.locale(detectedLocale);
  }, []);

  if (!wasLoaded) return null;

  return (
    <TypesafeI18n locale={detectedLocale}>
      <RouterProvider router={router} />
    </TypesafeI18n>
  );
};
