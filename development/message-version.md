We support cross tab syncing by adding a version tag, where client need to re-validate entire message on version change(except initial version `-1`).

The version tag is actually latest message id, but we must delete all message because: 

> Consider this case to understand why deletion of message less than id is not enough:
> *actor: A, B(two tabs)*
> 1. A open a tab, and created a message with id 1
> 2. LLM response with id 2
> 3. A created a message with id 3
> 4. LLM response with id 4
> 5. A disconnect
> 6. B delete message with id 3(four is deleted)
> 7. B created message with id 5 
> 8. LLM response with id 6
> 9. A connect, and found version to be 6.
