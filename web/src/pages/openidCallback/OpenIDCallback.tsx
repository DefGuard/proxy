import './style.scss';

import { LogoContainer } from '../../components/LogoContainer/LogoContainer';
import { PageContainer } from '../../shared/components/layout/PageContainer/PageContainer';
import { OpenIDCallbackCard } from './components/OpenIDCallbackCard';

export const OpenIDCallbackPage = () => {
  return (
    <PageContainer id="openidcallback-page">
      <LogoContainer />
      <OpenIDCallbackCard />
    </PageContainer>
  );
};
