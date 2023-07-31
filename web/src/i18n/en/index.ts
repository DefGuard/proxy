/* eslint-disable no-irregular-whitespace */
/* eslint-disable max-len */
import type { BaseTranslation } from '../i18n-types';

const en = {
  form: {
    errors: {
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

You have a time limit of **{time: string}** to complete this process.
If you have any questions, please consult your assigned admin.All necessary information can also be found at the bottom of the sidebar.`,
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
        back: 'Return to services menu',
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
      },
    },
  },
} satisfies BaseTranslation;

export default en;
