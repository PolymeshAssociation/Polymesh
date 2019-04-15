import React from 'react';
import {List, Icon, Button, Label, Popup} from 'semantic-ui-react';
import {ReactiveComponent} from 'oo7-react';
import {runtime, secretStore} from 'oo7-substrate';
import Identicon from 'polkadot-identicon';

export class SecretItem extends ReactiveComponent {
	constructor () {
		super()

		this.state = {
			display: null
		}
	}

	render () {
		let that = this
		let toggle = () => {
			let display = that.state.display
			if (display === null) {
				display = 'uri'
				window.setTimeout(() => that.setState({ display: null }), 5000)
				that.setState({ display })
			}
		}
		return this.state.display === 'uri'
			? <Label
				basic
				icon='privacy'
				onClick={toggle}
				content='URI '
				detail={this.props.uri}
			/>
			: <Popup trigger={<Icon
				circular
				className='eye slash'
				onClick={toggle}
			/>} content='Click to uncover seed/secret' />
	}
}

export class WalletList extends ReactiveComponent {
	constructor () {
		super([], {
			secretStore: secretStore(),
			shortForm: secretStore().map(ss => {
				let r = {}
				ss.keys.forEach(key => r[key.name] = runtime.indices.ss58Encode(runtime.indices.tryIndex(key.account)))
				return r
			})
		})
	}

	readyRender () {
		return <List divided verticalAlign='bottom' style={{padding: '0 0 4px 4px', overflow: 'auto', maxHeight: '20em'}}>{
			this.state.secretStore.keys.map(key =>
				<List.Item key={key.name}>
					<List.Content floated='right'>
						<SecretItem uri={key.uri}/>
						<Button size='small' onClick={() => secretStore().forget(key)}>Delete</Button>
					</List.Content>
					<List.Content floated='right'>
						<div>Crypto</div>
						<div style={{fontWeight: 'bold', width: '4em', color: key.type == 'sr25519' ? '#050' : '#daa'}}>
							{key.type == 'ed25519' ? 'Ed25519' : key.type == 'sr25519' ? 'Sr25519' : '???'}
						</div>
					</List.Content>
					<span className='ui avatar image' style={{minWidth: '36px'}}>
						<Identicon account={key.account} />
					</span>
					<List.Content>
						<List.Header>{key.name}</List.Header>
						<List.Description>
							{this.state.shortForm[key.name]}
						</List.Description>
					</List.Content>
				</List.Item>
			)
		}</List>
	}
}
