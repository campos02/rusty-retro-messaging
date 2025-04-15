import { defineAction, ActionError } from 'astro:actions';
import { z } from 'astro:schema';

export const server = {
  registration: defineAction({
    accept: 'form',
    input: z.object({
      email: z.string().email(),
      password: z.string(),
      password_confirmation: z.string(),
      code: z.string(),
    }),
    handler: async ({ email, password, password_confirmation, code }) => {
      const backendName = import.meta.env.BACKEND_NAME;
      const response = await fetch(`https://${backendName}/_r2m/register`, {
        method: "POST",
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          email: email,
          password: password,
          password_confirmation: password_confirmation,
          code: code
        }),
      });

      switch (response.status) {
        case 400:
          throw new ActionError({
            code: "BAD_REQUEST",
            message: await response.json()
          });
        case 401:
          throw new ActionError({
            code: "UNAUTHORIZED",
            message: await response.json()
          });
      }
    },
  })
}