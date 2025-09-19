import './style.scss';

import { m } from '../../paraglide/messages';
import { Logo } from '../../shared/components/Logo/Logo';
import { Page } from '../../shared/components/Page/Page';
import { SizedBox } from '../../shared/defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../shared/defguard-ui/types';
import { HomeChoice } from './components/HomeChoice';

const currentYear = new Date().getFullYear();

export const HomePage = () => {
  return (
    <Page id="home-page" variant="home">
      <SizedBox height={40} />
      <Logo />
      <SizedBox height={ThemeSpacing.Xl6} />
      <h1>{m.start_multi_title()}</h1>
      <SizedBox height={ThemeSpacing.Xl} />
      <p className="subtitle">{m.start_multi_subtitle()}</p>
      <SizedBox height={ThemeSpacing.Xl6} />
      <HomeChoice />
      <footer>
        <p>{m.start_footer_contact()}</p>
        <SizedBox height={ThemeSpacing.Xs} />
        <p>{m.start_footer_copyright({ currentYear: currentYear.toString() })}</p>
        <SizedBox height={ThemeSpacing.Sm} />
      </footer>
      <SizedBox height={28} />
    </Page>
  );
};
