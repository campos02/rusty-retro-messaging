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
  }),

  login: defineAction({
    accept: 'form',
    input: z.object({
      email: z.string().email(),
      password: z.string()
    }),
    handler: async ({ email, password }) => {
      const backendName = import.meta.env.PUBLIC_BACKEND_NAME;
      const response = await fetch(`https://${backendName}/_r2m/login`, {
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

  changeEmail: defineAction({
    accept: 'form',
    input: z.object({
      token: z.string(),
      current_email: z.string().email(),
      new_email: z.string().email(),
      password: z.string()
    }),
    handler: async ({ token, current_email, new_email, password }) => {
      const backendName = import.meta.env.PUBLIC_BACKEND_NAME;
      const response = await fetch(`https://${backendName}/_r2m/user/change-email`, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          current_email: current_email,
          new_email: new_email,
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
    },
  }),

  changePassword: defineAction({
    accept: 'form',
    input: z.object({
      token: z.string(),
      current_password: z.string(),
      new_password: z.string()
    }),
    handler: async ({ token, current_password, new_password }) => {
      const backendName = import.meta.env.PUBLIC_BACKEND_NAME;
      const response = await fetch(`https://${backendName}/_r2m/user/change-password`, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          current_password: current_password,
          new_password: new_password
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

  deleteAccount: defineAction({
    accept: 'form',
    input: z.object({
      token: z.string(),
      password: z.string()
    }),
    handler: async ({ token, password }) => {
      const backendName = import.meta.env.PUBLIC_BACKEND_NAME;
      const response = await fetch(`https://${backendName}/_r2m/user`, {
        method: "DELETE",
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
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
    },
  }),
}