import './style.scss';

import { m } from '../../../paraglide/messages';
import { ContainerWithIcon } from '../../../shared/components/ContainerWithIcon/ContainerWithIcon';
import { Page } from '../../../shared/components/Page/Page';
import { EnrollmentStep } from '../../../shared/components/Step/Step';
import { Button } from '../../../shared/defguard-ui/components/Button/Button';
import { Divider } from '../../../shared/defguard-ui/components/Divider/Divider';
import { Icon } from '../../../shared/defguard-ui/components/Icon';
import { SizedBox } from '../../../shared/defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../../shared/defguard-ui/types';

export const ConfigureClientPage = () => {
  return (
    <Page id="configure-client-page">
      <EnrollmentStep current={2} max={2} />
      <header>
        <h1>{m.client_setup_title()}</h1>
        <p>{m.client_setup_subtitle()}</p>
      </header>
      <SizedBox height={ThemeSpacing.Xl5} />
      <ContainerWithIcon icon="desktop" id="setup-desktop">
        <h5>{m.client_setup_desktop_title()}</h5>
        <div className="automatic">
          <p>{m.client_setup_desktop_auto_title()}</p>
          <p>
            {m.client_setup_desktop_auto_explain_1()}
            <span>{m.client_setup_desktop_auto_explain_2()}</span>
          </p>
        </div>
        <div className="buttons">
          <Button
            text={m.client_setup_desktop_auto_button_one_click()}
            variant="primary"
            iconRight="open-in-new-window"
          />
        </div>
        <Divider />
        <div className="foldable">
          <p className="title">{m.client_setup_desktop_manual_title()}</p>
          <p className="subtitle">{m.client_setup_desktop_manual_subtitle()}</p>
          <div className="fields"></div>
        </div>
        <button className="fold-button">
          <Icon icon="config" />
          <span>
            {m.client_setup_desktop_manual_fold({
              intent: m.controls_show(),
            })}
          </span>
        </button>
      </ContainerWithIcon>
      <SizedBox height={ThemeSpacing.Md} />
      <ContainerWithIcon id="setup-mobile" icon="mobile">
        <h5>{}</h5>
        <p></p>
        <div className="bottom">
          <div className="qr"></div>
          <div className="download">
            <p></p>
            <div className="links">
              <a target="blank" rel="noopener noreferrer">
                <Button
                  variant="outlined"
                  iconLeft="android"
                  text={m.client_setup_mobile_google()}
                />
              </a>
              <a target="blank" rel="noopener noreferrer">
                <Button
                  variant="outlined"
                  iconLeft="android"
                  text={m.client_setup_mobile_google()}
                />
              </a>
            </div>
          </div>
        </div>
      </ContainerWithIcon>
      <footer>
        <p>{m.client_setup_footer_extra()}</p>
        <p>{m.start_footer_contact()}</p>
      </footer>
    </Page>
  );
};
