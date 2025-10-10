import './style.scss';

import { m } from '../../paraglide/messages';
import { Page } from '../../shared/components/Page/Page';
import { SizedBox } from '../../shared/defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../shared/defguard-ui/types';
import { HomeChoice } from './components/HomeChoice';

const currentYear = new Date().getFullYear();

export const HomePage = () => {
  return (
    <Page id="home-page" variant="home">
      <h1>{m.start_multi_title()}</h1>
      <SizedBox height={ThemeSpacing.Xl} />
      <p className="subtitle">{m.start_multi_subtitle()}</p>
      <SizedBox height={ThemeSpacing.Xl6} />
      <HomeChoice />
      <footer>
        <p className="copyright">
          <span>Copyright ©2023-{currentYear.toString()} </span>
          <a href="https://defguard.net" target="_blank" rel="noopener noreferrer">
            Defguard
          </a>
          <span> Sp. z o.o.</span>
        </p>
      </footer>
    </Page>
  );
};
