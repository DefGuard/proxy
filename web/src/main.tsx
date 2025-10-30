import './shared/defguard-ui/scss/index.scss';

// import { TanStackDevtools } from '@tanstack/react-devtools';
// import { ReactQueryDevtoolsPanel } from '@tanstack/react-query-devtools';
// import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './app/App';

// biome-ignore lint/style/noNonNullAssertion: always there
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
    {/* <TanStackDevtools
      plugins={[
        {
          name: 'TanStack Router',
          render: <TanStackRouterDevtoolsPanel />,
        },
        {
          name: 'TanStack Query',
          render: <ReactQueryDevtoolsPanel />,
        },
      ]}
    /> */}
  </StrictMode>,
);
