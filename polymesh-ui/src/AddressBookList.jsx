import React from 'react';
import {List, Button} from 'semantic-ui-react';
import {ReactiveComponent} from 'oo7-react';
import {runtime, addressBook} from 'oo7-substrate';
import Identicon from 'polkadot-identicon';

export class AddressBookList extends ReactiveComponent {
	constructor () {
		super([], {
			addressBook: addressBook(),
			shortForm: addressBook().map(ss => {
				let r = {}
				ss.accounts.forEach(account => r[account.name] = runtime.indices.ss58Encode(runtime.indices.tryIndex(account.account)))
				return r
			})
		})
	}

	readyRender () {
		return <List divided verticalAlign='bottom' style={{padding: '0 0 4px 4px', overflow: 'auto', maxHeight: '20em'}}>{
			this.state.addressBook.accounts.map(account =>
				<List.Item key={account.name}>
					<List.Content floated='right'>
						<Button size='small' onClick={() => addressBook().forget(account)}>Delete</Button>
					</List.Content>
					<span className='ui avatar image' style={{minWidth: '36px'}}>
						<Identicon account={account.account} />
					</span>
					<List.Content>
						<List.Header>{account.name}</List.Header>
						<List.Description>
							{this.state.shortForm[account.name]}
						</List.Description>
					</List.Content>
				</List.Item>
			)
		}</List>
	}
}
