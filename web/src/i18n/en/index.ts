/* eslint-disable no-irregular-whitespace */
/* eslint-disable max-len */
import type { BaseTranslation } from '../i18n-types';

const en = {
  time: {
    seconds: {
      singular: 'second',
      prular: 'seconds',
    },
    minutes: {
      singular: 'minute',
      prular: 'minutes',
    },
  },
  form: {
    errors: {
      invalid: 'Field is invalid',
      email: 'Enter a valid E-mail',
      required: 'Field is required',
      minLength: 'Min length of {length: number}',
      maxLength: 'Max length of {length: number}',
      specialsRequired: 'At least one special character',
      specialsForbidden: 'Special characters are forbidden',
      numberRequired: 'At least one number required',
      password: {
        floatingTitle: 'Please correct the following:',
      },
      oneLower: 'At least one lower case character',
      oneUpper: 'At least one upper case character',
    },
  },
  common: {
    controls: {
      back: 'Back',
      next: 'Next',
      submit: 'Submit',
    },
  },
  components: {
    adminInfo: {
      title: 'Your admin',
    },
  },
  pages: {
    enrollment: {
      sideBar: {
        title: 'Enrollment',
        steps: {
          welcome: 'Welcome',
          verification: 'Data verification',
          password: 'Create password',
          vpn: 'Configure VPN',
          finish: 'Finish',
        },
        appVersion: 'Application version',
      },
      stepsIndicator: {
        step: 'Step',
        of: 'of',
      },
      timeLeft: 'Time left',
      steps: {
        welcome: {
          title: 'Hello, {name: string}',
          explanation: `
In order to gain access to the company infrastructure, we require you to complete this enrollment process. During this process, you will need to:

1. Verify your data
2. Create your password
3. Configurate VPN device

You have a time limit of **{time: string} minutes** to complete this process.
If you have any questions, please consult your assigned admin.All necessary information can be found at the bottom of the sidebar.`,
        },
        dataVerification: {
          title: 'Data verification',
          messageBox:
            'Please, check your data. If anything is wrong, notify your admin after you complete the process.',
          form: {
            fields: {
              firstName: {
                label: 'Name',
              },
              lastName: {
                label: 'Last name',
              },
              email: {
                label: 'E-mail',
              },
              phone: {
                label: 'Phone number',
              },
            },
          },
        },
        password: {
          title: 'Create password',
          form: {
            fields: {
              password: {
                label: 'Create new password',
              },
              repeat: {
                label: 'Confirm new password',
                errors: {
                  matching: `Passwords aren't matching`,
                },
              },
            },
          },
        },
        deviceSetup: {
          optionalMessage: `* This step is OPTIONAL. You can skip it if you wish. This can be configured later in defguard.`,
          cards: {
            device: {
              title: 'Configure your device for VPN',
              create: {
                submit: 'Create Configuration',
                messageBox:
                  'Please be advised that you have to download the configuration now, since we do not store your private key. After this dialog is closed, you will not be able to get your fulll configuration file (with private keys, only blank template).',
                form: {
                  fields: {
                    name: {
                      label: 'Device Name',
                    },
                    public: {
                      label: 'My Public Key',
                    },
                    toggle: {
                      generate: 'Generate key pair',
                      own: 'Use my own public key',
                    },
                  },
                },
              },
              config: {
                messageBox: {
                  auto: `
       <p>
          Please be advised that you have to download the configuration now,
          since <strong>we do not</strong> store your private key. After this
          dialog is closed, you <strong>will not be able</strong> to get your
          full configuration file (with private keys, only blank template).
        </p>
`,
                  manual: `
        <p>
          Please be advised that configuration provided here <strong> does not include private key and uses public key to fill it's place </strong> you will need to repalce it on your own for configuration to work properly.
        </p>
`,
                },
                deviceNameLabel: 'My Device Name',
                noNetworksMessage: "You don't have access to any networks",
                cardTitle:
                  'Use provided configuration file below by scanning QR Code or importing it as file on your devices WireGuard app.',
                card: {
                  selectLabel: 'Config file for location',
                },
              },
            },
            guide: {
              title: 'Quick Guide',
              messageBox: 'This quick guide will help you with device configuration.',
              step: 'Step {step: number}:',
              steps: {
                wireguard: {
                  content:
                    'Download and install WireGuard client on your compputer or app on phone.',
                  button: 'Download WireGuard',
                },
                downloadConfig: 'Download provided configuration file to your device.',
                addTunnel: `Open WireGuard and select "Add Tunnel" (Import tunnel(s) from file). Find your
Defguard configuration file and hit "OK". On phone use WireGuard app “+” icon and scan QR code.`,
                activate: 'Select your tunnel from the list and press "activate".',
                finish: `
**Great work - your Defguard VPN is now active!**

If you want to disengage your VPN connection, simply press "deactivate".
`,
              },
            },
          },
        },
        finish: {
          title: 'Configuration completed!',
        },
      },
    },
    resetPassword: {
      steps: {
        email: {
          controls: {
            send: 'Send',
          },
          title: 'Enter your e-mail',
          form: {
            fields: {
              email: {
                label: 'E-mail',
              },
            },
          },
        },
        linkSent: {
          controls: {
            back: 'Back to main menu',
          },
          messageBox:
            'If the provided email address is assigned to any account - the password reset will be initiated and you will recieve an email with further istructions.',
        },
        securityCode: {
          controls: {
            sendCode: 'Send code',
          },
          title: 'Enter security code',
          messagebox:
            'In order to reset the password, please provide security code that will be sent to your number:',
          form: {
            fields: {
              code: {
                label: 'Security code',
              },
            },
          },
        },
        resetPassword: {
          title: 'Reset your password',
          controls: {
            submit: 'Reset password',
          },
          form: {
            errors: {
              repeat: "Fields doesn't match",
            },
            fields: {
              password: {
                label: 'New password',
              },
              repeat: {
                label: 'Confirm password',
              },
            },
          },
        },
        resetSuccess: {
          controls: {
            back: 'Return to services menu',
          },
          messageBox: 'Congratulations, your new password is set.',
        },
        resetFailed: {
          controls: {
            back: 'Return to start',
          },
          messageBox:
            'The entered code is invalid. Please start the process from the beginning.',
        },
      },
    },
    sessionTimeout: {
      card: {
        header: 'Session timed out',
        message:
          'Sorry, you have exceeded the time limit to complete the process. Please try again. If you need assistance, please watch our guide or contact your administrator.',
      },
      controls: {
        back: 'Enter new token',
        contact: 'Contact admin',
      },
    },
    token: {
      card: {
        title: 'Please, enter your personal enrollment token',
        messageBox: {
          email: 'You can find token in e-mail message or use direct link.',
        },
        form: {
          errors: {
            token: {
              required: 'Token is required',
            },
          },
          fields: {
            token: {
              placeholder: 'Token',
            },
          },
          controls: {
            submit: 'Next',
          },
        },
        oidc: {
          oidcButton: 'Sign in with',
          title: 'Or Sign In with External SSO',
          infoBox:
            'If you would like to initiate the enrollment process using External SSO, please click the link below to sign in and start the process.',
        },
      },
    },
    oidcLogin: {
      card: {
        title: 'Start your enrollment process',
        infoBox:
          'Thank you for validating your account, please follow instruction below for configuring your VPN connection.',
        steps: {
          first: 'Please download and install defguard VPN Desktop Client.',
          second: `2. Open the client and <i>Add Instance</i>. Copy the data provided below into the corresponding fields. 
              You can also learn more about the process in our <a
              href="https://docs.defguard.net/help/configuring-vpn/add-new-instance"
              target="_blank"
              >
              documentation
              </a>.`,
          tokenInput: {
            instanceUrl: 'Instance URL',
            token: 'Token',
            title: 'Please provide instance URL and token',
            addInstance: 'Add Instance',
          },
        },
      },
    },
  },
} satisfies BaseTranslation;

export default en;
