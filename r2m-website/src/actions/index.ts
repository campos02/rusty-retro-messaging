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
      const backendName = import.meta.env.PUBLIC_BACKEND_NAME;
      let url;
      if (import.meta.env.DEV)
        url = `http://${backendName}/_r2m/register`;
      else
        url = `https://${backendName}/_r2m/register`;

      const response = await fetch(url, {
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
  }),

  login: defineAction({
    accept: 'form',
    input: z.object({
      email: z.string().email(),
      password: z.string()
    }),
    handler: async ({ email, password }) => {
      const backendName = import.meta.env.PUBLIC_BACKEND_NAME;
      let url;
      if (import.meta.env.DEV)
        url = `http://${backendName}/_r2m/login`;
      else
        url = `https://${backendName}/_r2m/login`;

      const response = await fetch(url, {
        method: "POST",
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          email: email,
          password: password
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

      return await response.json();
    },
  }),

  logout: defineAction({
    handler: async (context) => {
      const backendName = import.meta.env.PUBLIC_BACKEND_NAME;
      const token = context.cookies.get('user_token').value;

      await fetch(`https://${backendName}/_r2m/logout`, {
        headers: {
          Authorization: `Bearer ${token}`
        }
      });

      context.cookies.delete('user_token');
    }
  }),

  user: defineAction({
    handler: async (context) => {
      const backendName = import.meta.env.PUBLIC_BACKEND_NAME;
      const token = context.cookies.get('user_token').value;

      const response = await fetch(`https://${backendName}/_r2m/user`, {
        headers: {
          Authorization: `Bearer ${token}`
        }
      });

      return await response.json();
    }
  }),
}