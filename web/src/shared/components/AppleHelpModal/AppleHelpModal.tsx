import './style.scss';
import { m } from '../../../paraglide/messages';
import { Modal } from '../../defguard-ui/components/Modal/Modal';
import { ModalControls } from '../../defguard-ui/components/ModalControls/ModalControls';
import { SizedBox } from '../../defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../defguard-ui/types';
import apple_video_src from './assets/apple_hardware_help.mp4';

type Props = {
  isOpen: boolean;
  onClose: () => void;
};

export const AppleHelpModal = ({ isOpen, onClose }: Props) => {
  return (
    <Modal
      title={m.client_download_apple_help_title()}
      size="small"
      isOpen={isOpen}
      onClose={onClose}
      id="apple-hardware-help"
    >
      <p>{m.client_download_apple_help_content_1()}</p>
      <SizedBox height={ThemeSpacing.Xl} />
      <video autoPlay loop playsInline preload="auto">
        <source src={`${apple_video_src}#t=0.1`} type="video/mp4" />
      </video>
      <SizedBox height={ThemeSpacing.Xl} />
      <p>{m.client_download_apple_help_content_2()}</p>
      <ModalControls
        submitProps={{
          text: m.controls_got_it(),
          onClick: onClose,
        }}
      />
    </Modal>
  );
};
