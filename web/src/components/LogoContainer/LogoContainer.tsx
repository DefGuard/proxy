import './style.scss';

import { Divider } from '../../shared/components/layout/Divider/Divider';
import { DividerDirection } from '../../shared/components/layout/Divider/types';
import SvgDefguardLogoText from '../../shared/components/svg/DefguardLogoText';
import SvgTeoniteLogo from '../../shared/components/svg/TeoniteLogo';

export const LogoContainer = () => {
  return (
    <div className="logo-container">
      <a href="https://defguard.net/" target="_blank" rel="noreferrer">
        <SvgDefguardLogoText className="defguard" />
      </a>
      <Divider direction={DividerDirection.VERTICAL} />
      <a href="https://teonite.com" target="_blank" rel="noreferrer">
        <SvgTeoniteLogo className="teonite" />
      </a>
    </div>
  );
};
