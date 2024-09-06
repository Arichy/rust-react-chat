import { Checkbox, TextInput } from '@mantine/core';
import { BaseError, User } from '@types';
import { Dispatch, SetStateAction, useEffect, useState } from 'react';
import { ActionFunctionArgs, redirect, useFetcher, useNavigate } from 'react-router-dom';
import { z } from 'zod';
import * as _ from 'radash';

const ACTION_TYPES = {
  SIGNUP: 'signup',
  SIGNIN: 'signin',
};

type ActionData = { status: number; data: BaseError | User; sign_in?: boolean };

const signupSchema = z
  .object({
    type: z.enum([ACTION_TYPES.SIGNUP, ACTION_TYPES.SIGNIN]),
    username: z.string(),
    password: z.string(),
    signin: z
      .string()
      .optional()
      .transform(val => (val === 'on' ? true : false)),
  })
  .transform(data => ({
    ..._.omit(data, ['signin']),
    sign_in: data.signin,
  }));

export async function action({ request }: ActionFunctionArgs): Promise<ActionData | Response> {
  const formData = await request.formData();
  const formDataObj = Object.fromEntries(formData);

  const result = signupSchema.safeParse(formDataObj);
  if (!result.success) {
    return {
      status: 400,
      data: {
        success: false,
        message: 'Invalid form data',
      },
      sign_in: false,
    };
  }

  switch (result.data.type) {
    case ACTION_TYPES.SIGNUP: {
      const res = await fetch('/api/auth/signup', {
        method: 'post',
        body: JSON.stringify(result.data),
        headers: {
          'Content-Type': 'application/json',
        },
      });

      return {
        status: res.status,
        data: await res.json(),
        sign_in: result.data.sign_in,
      };
    }
    case ACTION_TYPES.SIGNIN: {
      const res = await fetch('/api/auth/signin', {
        method: 'post',
        body: JSON.stringify(result.data),
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (res.status === 200) {
        return redirect('/');
      }

      return {
        status: res.status,
        data: await res.json(),
      };
    }
  }
  throw new Error('Invalid action type');
}

const FormSignUp = ({ setShowSignIn }: { setShowSignIn: Dispatch<SetStateAction<boolean>> }) => {
  const navigate = useNavigate();

  const fetcher = useFetcher<ActionData>();
  useEffect(() => {
    if (!fetcher.data) {
      return;
    }

    const { status, sign_in } = fetcher.data;
    if (status === 200) {
      if (sign_in) {
        console.log('redirecting');
        navigate('/');
      }
    }
  }, [fetcher.data, navigate]);

  return (
    <fetcher.Form method="post" className="mt-4 space-y-2">
      <TextInput
        label="Username"
        required
        type="text"
        name="username"
        radius="md"
        placeholder="Username"
        size="md"
        autoComplete="off"
        error={fetcher.data && fetcher.data.data && 'message' in fetcher.data.data ? fetcher.data?.data.message : null}
      />

      <TextInput
        label="Password"
        required
        type="password"
        name="password"
        radius="md"
        placeholder="Password"
        size="md"
        autoComplete="off"
      />

      <Checkbox name="signin" defaultChecked label="Sign in after success" />
      <input type="hidden" name="type" value={ACTION_TYPES.SIGNUP} />

      <div className="flex items-baseline justify-between">
        <button type="submit" className="px-6 py-2 mt-4 text-white bg-violet-600 rounded-lg hover:bg-violet-700 w-full">
          Submit
        </button>
      </div>
      <div className="pt-2 space-y-2 text-center">
        <p className="text-base text-gray-700">
          Already have a username?{' '}
          <button onClick={() => setShowSignIn(true)} className="text-violet-700 font-light">
            Sign In
          </button>
        </p>
      </div>
    </fetcher.Form>
  );
};

const FormSignIn = ({ setShowSignIn }: { setShowSignIn: Dispatch<SetStateAction<boolean>> }) => {
  const fetcher = useFetcher();

  return (
    <fetcher.Form method="post" className="mt-4 space-y-2">
      <TextInput
        label="Username"
        required
        type="text"
        name="username"
        radius="md"
        placeholder="Username"
        size="md"
        autoComplete="off"
        error={fetcher.data && fetcher.data.data && 'message' in fetcher.data.data ? fetcher.data?.data.message : null}
      />

      <TextInput
        label="Password"
        required
        type="password"
        name="password"
        radius="md"
        placeholder="Password"
        size="md"
        autoComplete="off"
      />
      <input type="hidden" name="type" value={ACTION_TYPES.SIGNIN} />

      <div className="flex items-baseline justify-between">
        <button type="submit" className="px-6 py-2 mt-4 text-white bg-violet-600 rounded-lg hover:bg-violet-700 w-full">
          Submit
        </button>
      </div>
      <div className="pt-2 space-y-2 text-center">
        <p className="text-base text-gray-700">
          Don't have username?{' '}
          <button onClick={() => setShowSignIn(false)} className="text-violet-700 font-light">
            Create
          </button>
        </p>
      </div>
    </fetcher.Form>
  );
};

export default function Auth() {
  const [isShowSigIn, setShowSignIn] = useState(true);

  return (
    <div className="bg-gradient-to-b from-orange-400 to-rose-400  min-h-screen flex items-center justify-center">
      <div className="px-8 py-6 mt-4 text-left bg-white  max-w-[400px] w-full rounded-xl shadow-lg">
        <h3 className="text-xl text-slate-800 font-semibold">{isShowSigIn ? 'Log in' : 'Create your account'}</h3>
        {isShowSigIn ? <FormSignIn setShowSignIn={setShowSignIn} /> : <FormSignUp setShowSignIn={setShowSignIn} />}
      </div>
    </div>
  );
}
