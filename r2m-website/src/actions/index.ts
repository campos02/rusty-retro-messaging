import { defineAction } from 'astro:actions';
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
        await fetch(`https://${backendName}/_r2m/register`, {
            method: "POST",
            body: JSON.stringify({
                email: email,
                password: password,
                password_confirmation: password_confirmation,
                code: code
            }),
        });
    },
  })
}